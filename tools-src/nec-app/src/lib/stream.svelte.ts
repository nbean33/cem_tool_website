// Collects streamed sweep points. Points of one job arrive in completion order and are kept
// sorted by index; the first point of a newer job replaces the old set, so a re-run never
// blanks the chart while it waits.
import type { SweepEvent } from "./types";

export class PointStream {
  job = 0;
  // Raw: frames carry large arrays that need no deep reactivity; every change reassigns.
  points = $state.raw<SweepEvent[]>([]);

  accept(e: SweepEvent) {
    if (e.job < this.job) return;
    if (e.job > this.job) {
      this.job = e.job;
      this.points = [];
    }
    const points = this.points.slice();
    let i = points.length;
    while (i > 0 && points[i - 1].index > e.index) i--;
    points.splice(i, 0, e);
    this.points = points;
  }

  clear() {
    this.points = [];
  }
}
