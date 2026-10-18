import { describe, expect, it } from "vitest";

import { display, episode_title } from "./displayElement.js";

const shows = {
  3: { name: "Heimat", poster_path: "/heimat.jpg", countries: ["DE"] },
};

const episode = (media) => ({
  filepath: "D:\\tv\\heimat\\s01e02.mkv",
  media: { kind: "episode", show_id: 3, ...media },
});

describe("display", () => {
  it("shows a film from its own fields", () => {
    const film = {
      filepath: "D:/films/alien.mkv",
      media: {
        kind: "film",
        title: "Alien",
        original_title: "Alien - Das Original",
        poster_path: "/alien.jpg",
        release_date: "1979-05-25",
        director: "Ridley Scott",
        countries: ["US", "GB"],
      },
    };
    expect(display(film, shows)).toEqual({
      kind: "film",
      title: "Alien",
      original_title: "Alien - Das Original",
      sort_title: "Alien",
      image: "/alien.jpg",
      date: "1979-05-25",
      director: "Ridley Scott",
      countries: ["US", "GB"],
    });
  });

  it("composes an episode title from its show", () => {
    const shown = display(
      episode({
        season: 1,
        episode: 2,
        name: "Die Mitte der Welt",
        still_path: "/still.jpg",
        air_date: "1984-09-19",
      }),
      shows
    );
    expect(shown.kind).toBe("episode");
    expect(shown.title).toBe("Heimat S01E02 Die Mitte der Welt");
    expect(shown.original_title).toBe(shown.title);
    expect(shown.sort_title).toBe(shown.title);
    expect(shown.image).toBe("/still.jpg");
    expect(shown.date).toBe("1984-09-19");
    expect(shown.countries).toEqual(["DE"]);
  });

  it("falls back to the show's poster for an episode without a still", () => {
    expect(display(episode({ still_path: null }), shows).image).toBe(
      "/heimat.jpg"
    );
  });

  it("shows an episode whose show is missing from the table", () => {
    const element = episode({ show_id: 99, season: 1, episode: 2, name: "N" });
    for (const table of [shows, {}, undefined]) {
      const shown = display(element, table);
      expect(shown.title).toBe("S01E02 N");
      expect(shown.image).toBeUndefined();
      expect(shown.countries).toBeUndefined();
    }
  });

  it("leaves out a missing season or episode number", () => {
    expect(display(episode({ name: "N" }), shows).title).toBe("Heimat N");
    expect(display(episode({ season: 0, name: "N" }), shows).title).toBe(
      "Heimat S00 N"
    );
    expect(display(episode({ episode: 12 }), shows).title).toBe("Heimat E12");
  });

  it("sorts an episode with nothing to show by its filename", () => {
    const shown = display(episode({ show_id: 99 }), shows);
    expect(shown.title).toBe("");
    expect(shown.sort_title).toBe("s01e02.mkv");
  });

  it("shows a show tile from its show", () => {
    const tile = { kind: "show", show_id: 3, filepath: "D:/tv/heimat/s01e01.mkv" };
    expect(display(tile, shows)).toMatchObject({
      kind: "show",
      title: "Heimat",
      original_title: "Heimat",
      sort_title: "Heimat",
      image: "/heimat.jpg",
      countries: ["DE"],
    });
    // Show missing from the table: sorted by its first file
    expect(display(tile, {}).sort_title).toBe("s01e01.mkv");
  });

  it("titles an episode inside its season", () => {
    expect(episode_title({ season: 1, episode: 2, name: "N" })).toBe("S01E02 · N");
    expect(episode_title({ season: 1, episode: 2 })).toBe("S01E02");
    expect(episode_title({ name: "N" })).toBe("N");
  });

  it("shows an unmatched file by its filename", () => {
    const expected = { kind: "unmatched", sort_title: "clip.mp4" };
    expect(
      display({ filepath: "D:/x/clip.mp4", media: { kind: "unmatched" } })
    ).toEqual(expected);
    // No media at all is treated as unmatched
    expect(display({ filepath: "D:/x/clip.mp4" })).toEqual(expected);
  });

  it("takes the filename from a path with backslashes", () => {
    const sort_title = (filepath) => display({ filepath }).sort_title;
    expect(sort_title("D:\\films\\sub\\clip.mp4")).toBe("clip.mp4");
    expect(sort_title("D:\\films/sub\\clip.mp4")).toBe("clip.mp4");
    expect(sort_title("clip.mp4")).toBe("clip.mp4");
  });
});
