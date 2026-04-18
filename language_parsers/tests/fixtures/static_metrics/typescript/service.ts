import { readFile } from "fs";

export function render(flag: boolean, items: number[]) {
  // render branch
  if (flag) {
    return items.length;
  }
  return 0;
}

export class Service {
  run(value: number) {
    for (const item of [value]) {
      if (item > 0) {
        return item;
      }
    }
    return 0;
  }
}
