# Runner setup
> To set up a runner, install RayRF on the machine, license it against your account, open the runner's TCP port, and start the daemon. See [Remote running](docs-remote-overview.html) for submitting jobs from the desktop app.

Runner licensing is seatless. A Pro or Founding license key from the dashboard licenses page validates the runner against your account and signs a machine-bound token for every job, without consuming an activation seat.

## Linux

Install once, with the license key from the dashboard licenses page:

```
./RayRF-Studio-<version>-x86_64.AppImage serve --install --license-key RAYRF-XXXX-XXXX-XXXX-XXXX
```

Then start serving:

```
./RayRF-Studio-<version>-x86_64.AppImage serve
```

The runner validates the key, registers this machine's address to your account, and your other machines discover it automatically. The runner exposes every device the box can run on: the CPU when the CPU backend is present and each NVIDIA GPU when the CUDA backend answers. Pass `--devices` with `--install` to expose a subset, for example `--devices cuda:0,cuda:1` on a box whose CPU should stay out of the pool. Later starts are the same plain `serve`, because the configuration is stored, and the runner window lets you check devices on and off while it serves.

Open the runner port so clients can reach it. Jobs move over a direct link to port 8787, which a host firewall or cloud security group often blocks by default. On a box running `ufw` that is `sudo ufw allow 8787/tcp`. On a cloud VM also open the port in the provider's security group.

To keep the runner alive across reboots, pass `--autostart` with `--install`. It writes a systemd unit into the state dir and prints the install command:

```
sudo cp /var/lib/rayrf/rayrf-runner.service /etc/systemd/system/ && sudo systemctl enable --now rayrf-runner
```

## Windows

The same two steps, from the install folder (`%LOCALAPPDATA%\Programs\RayRF Studio`). `rayrf` resolves to the console launcher `rayrf.com`, which waits and streams output, and `RayRF.exe` is the windowed app.

```
.\rayrf serve --install --license-key RAYRF-XXXX-XXXX-XXXX-XXXX
```

```
.\rayrf serve
```

Open the runner port so clients can reach it, the same as on Linux. Windows Firewall blocks inbound connections by default, so a runner that registers with your account is still unreachable until port 8787 is allowed. From an Administrator PowerShell:

```
netsh advfirewall firewall add rule name="RayRF runner 8787" dir=in action=allow protocol=TCP localport=8787
```

On a cloud VM also open the port in the provider's security group. If the machine's network is set to Public, allow the rule on the Public profile (the command above covers all profiles).

To run at startup, register a Scheduled Task (or a service wrapper) that runs `RayRF.exe serve --state-dir "C:\ProgramData\RayRF\runner"` from the install folder. Passing `--autostart` with `--install` prints the serve command with the resolved state dir.

## Advertise a fixed address

By default the runner advertises its active LAN addresses and the public IP observed by the website. Clients use an address that passes the verified connection. No files or results pass through the website.

For a translated port or fixed hostname, set the reachable endpoint with `--advertise HOST:PORT` at install or serve time: on a NAT or portmapped host (a rented container, a port-forwarded home box) that is the external address and mapped port, which can differ from the listen port. Registration is metadata only. Jobs move over a direct TLS 1.3 link to the advertised address, so the port (default 8787) must be reachable through the firewall or cloud security group.

`--advertise off` skips account registration entirely. The runner then serves direct connections only, and clients add it by host, port, and fingerprint.

## What each piece does

- `--install` writes `runner.json` (name, owner, host, port, slots, devices, advertise, and the license key when one is resolved) and the identity into the state dir. Command-line flags always win over the stored values.
- The **state dir** holds the identity, the configuration, and the durable job queue. It resolves from `--state-dir`, then the `RAYRF_STATE_DIR` environment variable, then the machine default (`C:\ProgramData\RayRF\runner` on Windows, `/var/lib/rayrf` on Linux when writable, else `~/.rayrf/runner`). Keep every invocation on one state dir: a new dir mints a new identity, and clients that pinned the old fingerprint refuse the new one.
- `--devices` names the devices jobs may run on, as a comma-separated list of `cpu` and `cuda:<index>`. Omitted, every device the box can run on is exposed. A job names one of them or `auto`, which takes the first free device with GPUs first. A named device queues the job until it is free. One job runs per device at a time.
- Control belongs to the license key's account: a desktop signed in to the same account proves it with its signed token and gets submit and control, and every other connection is view-only. `--owner` sets the account the runner displays, and `--open-owner` grants control to any peer on a trusted LAN. `--allow-unlicensed` skips the license gate for closed-network development. In that mode `--owner` is trusted by the email a peer types, and the start banner says so.
- `--slots` caps how many jobs run at once below the one-per-device default.

## Verify and connect

Account discovery verifies the advertised identity automatically. For a direct connection, compare the fingerprint printed by `--install` with the client prompt, or supply it explicitly:

```
rayrf serve --health                                status JSON on the box
rayrf remote info --host HOST --fingerprint FP      from a client
```

A client that connects by CLI stores the fingerprint it first trusted in `~/.rayrf/pins.json` (`%USERPROFILE%\.rayrf\pins.json` on Windows). If a runner's key later changes (wiping its state dir mints a new identity), the next connection is refused. After verifying the new fingerprint out of band, reconnect with `--fingerprint NEWFP`, or remove that runner's `host:port` entry from the pins file and reconnect to trust it fresh. For a saved direct desktop connection, remove it and add the verified new identity. Account discovery uses the current advertisement automatically.

The runner also has a windowed form, `rayrf serve --window`, which is what the desktop app's "Use this computer as a runner" button starts: the same daemon with the runner name, discovery status, device switches, and job counts. Address, identity, and logs are under Details.

Submit work from the desktop app ([Remote running](docs-remote-overview.html)) or from the CLI with the `rayrf remote` verbs ([CLI reference](docs-cli-reference.html)).

## Updating a runner

A runner and the desktop app or CLI that connects to it must run the same RayRF release. A mismatch is refused with a message that says which end to update. To update a runner, stop `serve`, install the new release on the runner machine, and start `serve` again on the same state dir. The stored configuration and identity carry over, so clients keep their pinned fingerprint.
