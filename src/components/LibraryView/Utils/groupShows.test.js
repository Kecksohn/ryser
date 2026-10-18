import { describe, expect, it } from "vitest";

import {
  all_watched,
  date_added,
  episode_files,
  group_library,
  group_seasons,
  is_watched,
  owned_label,
  season_files,
  watched_label,
} from "./groupShows.js";

const film = {
  filepath: "D:/films/alien.mkv",
  watched: true,
  length_in_seconds: 7000,
  timestamp_modified: 50,
  media: { kind: "film", title: "Alien" },
};
const unmatched = {
  filepath: "D:/films/clip.mp4",
  watched: false,
  media: { kind: "unmatched" },
};

const episode = (show_id, season, number, extra = {}) => ({
  filepath: `D:/tv/${show_id}/s${season}e${number}.mkv`,
  watched: false,
  length_in_seconds: 100,
  timestamp_modified: 10,
  ...extra,
  media: { kind: "episode", show_id, season, episode: number, ...extra.media },
});

const show = {
  name: "Heimat",
  poster_path: "/heimat.jpg",
  seasons: [
    { season_number: 0, name: "Specials", episode_count: 3 },
    { season_number: 1, name: "Season 1", air_date: "1984-09-16", episode_count: 11 },
    { season_number: 2, poster_path: "/s2.jpg", episode_count: 13 },
  ],
};

const numbers = (files) => files.map((file) => file.media.episode);

describe("group_library", () => {
  it("keeps films and unmatched files and puts a show's episodes into one tile", () => {
    const e1 = episode(3, 1, 1);
    const e2 = episode(3, 1, 2);
    const other = episode(4, 1, 1);
    const tiles = group_library([e2, film, other, unmatched, e1]);

    expect(tiles.map((tile) => tile.kind ?? tile.media.kind)).toEqual([
      "show",
      "film",
      "show",
      "unmatched",
    ]);
    expect(tiles[0]).toMatchObject({ show_id: 3, episodes: [e2, e1] });
    expect(tiles[2]).toMatchObject({ show_id: 4, episodes: [other] });
    expect(tiles[1]).toBe(film);
  });

  it("gives a show tile the fields the library sort reads", () => {
    const [tile] = group_library([
      episode(3, 1, 2, { timestamp_modified: 30 }),
      episode(3, 1, 1, { timestamp_modified: 70 }),
    ]);
    expect(tile.filepath).toBe("D:/tv/3/s1e1.mkv");
    expect(tile.length_in_seconds).toBe(200);
    expect(tile.timestamp_modified).toBe(70);
  });
});

describe("group_seasons", () => {
  const files = [
    episode(3, 0, 2),
    episode(3, 2, 1),
    episode(3, 1, 10),
    episode(3, 1, 2),
    episode(3, 1, null, { filepath: "D:/tv/3/making of.mkv" }),
    episode(3, null, null, { filepath: "D:/tv/3/b-trailer.mkv" }),
    episode(3, null, null, { filepath: "D:/tv/3/a-interview.mkv" }),
  ];

  it("orders seasons by number with specials last", () => {
    const { seasons } = group_seasons(show, files);
    expect(seasons.map((season) => season.season_number)).toEqual([1, 2, 0]);
  });

  it("orders episodes by number and keeps files without number apart", () => {
    const { seasons, extras } = group_seasons(show, files);
    expect(numbers(seasons[0].episodes)).toEqual([2, 10]);
    expect(seasons[0].extras.map((file) => file.filepath)).toEqual([
      "D:/tv/3/making of.mkv",
    ]);
    // Files without season belong to the show, by filename
    expect(extras.map((file) => file.filepath)).toEqual([
      "D:/tv/3/a-interview.mkv",
      "D:/tv/3/b-trailer.mkv",
    ]);
    expect(season_files(seasons[0])).toHaveLength(3);
  });

  it("takes name, poster and year from the show's season list", () => {
    const { seasons } = group_seasons(show, files);
    expect(seasons[0]).toMatchObject({
      name: "Season 1",
      year: "1984",
      poster_path: "/heimat.jpg",
      episode_count: 11,
    });
    expect(seasons[1].poster_path).toBe("/s2.jpg");
  });

  it("falls back to a generic name and the earliest owned episode's year", () => {
    const { seasons } = group_seasons({ seasons: [] }, [
      episode(3, 2, 2, { media: { air_date: "1993-02-01" } }),
      episode(3, 2, 1, { media: { air_date: "1992-12-30" } }),
      episode(3, 2, 3),
      episode(3, 0, 1),
    ]);
    expect(seasons[0]).toMatchObject({ name: "Season 2", year: "1992" });
    expect(seasons[0].episode_count).toBeUndefined();
    expect(seasons[1]).toMatchObject({ name: "Specials", year: undefined });
  });

  it("works without the show in the show table", () => {
    const { seasons } = group_seasons(undefined, [episode(3, 1, 1)]);
    expect(seasons[0]).toMatchObject({ name: "Season 1", poster_path: undefined });
  });
});

describe("derived state", () => {
  const watched = episode(3, 1, 1, { watched: true });
  const unwatched = episode(3, 1, 2);

  it("a season or show is watched when all its files are", () => {
    expect(all_watched([watched])).toBe(true);
    expect(all_watched([watched, unwatched])).toBe(false);
    expect(all_watched([])).toBe(false);
  });

  it("counts watched episodes and extras apart", () => {
    const extra = episode(3, null, null, { watched: true });
    const season_extra = episode(3, 1, null);
    expect(watched_label([watched, unwatched, episode(3, 1, 3)])).toBe(
      "Watched 1/3"
    );
    expect(watched_label([watched, unwatched, extra, season_extra])).toBe(
      "Watched 1/2, Extras: 1/2"
    );
    expect(episode_files([watched, extra])).toEqual([watched]);
    // Nothing numbered: the files count as the episodes
    expect(watched_label([extra, season_extra])).toBe("Watched 1/2");
    expect(episode_files([extra])).toEqual([extra]);
  });

  it("counts owned episodes against TMDB's episode count", () => {
    const season = {
      episode_count: 10,
      // Two files of episode 2 count once, extras not at all
      episodes: [episode(3, 1, 1), episode(3, 1, 2), episode(3, 1, 2)],
      extras: [episode(3, 1, null)],
    };
    expect(owned_label(season)).toBe("have 2 of 10");
    expect(owned_label({ ...season, episode_count: undefined })).toBeNull();
  });

  it("a show counts as unwatched while any episode is", () => {
    const [show_tile] = group_library([watched, unwatched]);
    expect(is_watched(show_tile)).toBe(false);
    unwatched.watched = true;
    expect(is_watched(show_tile)).toBe(true);
    // An unwatched extra does not make the show unwatched
    const [with_extra] = group_library([watched, episode(3, null, null)]);
    expect(is_watched(with_extra)).toBe(true);
    expect(is_watched(film)).toBe(true);
    expect(is_watched(unmatched)).toBe(false);
  });

  it("a show's date added is its newest file", () => {
    expect(
      date_added([
        episode(3, 1, 1, { timestamp_modified: 5 }),
        episode(3, 1, 2, { timestamp_modified: 9 }),
      ])
    ).toBe(9);
  });
});
