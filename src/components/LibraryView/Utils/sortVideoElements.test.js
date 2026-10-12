import { describe, expect, it, vi } from "vitest";

import { sort_video_elements } from "./sortVideoElements.js";

const shows = { 3: { name: "Heimat" } };

const film = {
  filepath: "D:\\b\\zzz.mkv",
  media: { kind: "film", title: "Alien" },
  length_in_seconds: 7000,
  timestamp_modified: 300,
};
const episode = {
  filepath: "D:\\a\\e.mkv",
  media: { kind: "episode", show_id: 3, season: 1, episode: 2 },
  length_in_seconds: 3000,
  timestamp_modified: 100,
};
const unmatched = {
  filepath: "D:\\c\\Blade.mkv",
  media: { kind: "unmatched" },
  length_in_seconds: 5000,
  timestamp_modified: 200,
};
const elements = [episode, unmatched, film];

function sort(order, last_sort_order = "filepath") {
  const set_last_sort_order = vi.fn();
  const sorted = sort_video_elements(
    elements,
    order,
    last_sort_order,
    set_last_sort_order,
    shows
  );
  return { sorted, set_last_sort_order };
}

describe("sort_video_elements", () => {
  it("orders films, episodes and unmatched files by what is shown", () => {
    // "Alien", "Blade.mkv" (filename), "Heimat S01E02"
    const { sorted, set_last_sort_order } = sort("title");
    expect(sorted).toEqual([film, unmatched, episode]);
    expect(set_last_sort_order).toHaveBeenCalledWith("title");
  });

  it("reverses when called again with the same order", () => {
    const { sorted, set_last_sort_order } = sort("title", "title");
    expect(sorted).toEqual([episode, unmatched, film]);
    expect(set_last_sort_order).toHaveBeenCalledWith("title_reverse");
  });

  it("orders by duration, shortest first", () => {
    expect(sort("duration").sorted).toEqual([episode, unmatched, film]);
    expect(sort("duration", "duration").sorted).toEqual([
      film,
      unmatched,
      episode,
    ]);
  });

  it("orders by timestamp, newest first", () => {
    expect(sort("timestamp").sorted).toEqual([film, unmatched, episode]);
    expect(sort("timestamp", "timestamp").sorted).toEqual([
      episode,
      unmatched,
      film,
    ]);
  });

  it("orders by filepath", () => {
    const { sorted, set_last_sort_order } = sort("filepath", "title");
    expect(sorted).toEqual([episode, film, unmatched]);
    expect(set_last_sort_order).toHaveBeenCalledWith("filepath");
    expect(sort("filepath", "filepath").sorted).toEqual([
      unmatched,
      film,
      episode,
    ]);
  });

  it("does not reorder the array it was given", () => {
    sort("title");
    expect(elements).toEqual([episode, unmatched, film]);
  });
});
