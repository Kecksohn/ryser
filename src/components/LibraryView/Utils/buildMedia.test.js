import { describe, expect, it } from "vitest";

import { build_media } from "./buildMedia.js";

// Also deserialized by the Rust tests in video_element.rs
import film from "../../../../test-fixtures/media/film.json";
import user_film from "../../../../test-fixtures/media/user_film.json";
import episode from "../../../../test-fixtures/media/episode.json";
import episode_moved from "../../../../test-fixtures/media/episode_moved.json";
import episode_from_film from "../../../../test-fixtures/media/episode_from_film.json";
import film_from_episode from "../../../../test-fixtures/media/film_from_episode.json";
import extra from "../../../../test-fixtures/media/extra.json";

// What the episode form shows for the fixture episode
const episode_inputs = {
  title: "Die Mitte der Welt",
  image: "/heimat-s01e02.jpg",
  countries: [""],
};

const film_inputs = {
  title: "Alien",
  original_title: "Alien",
  countries: ["US", "GB"],
  image: "/alien-poster.jpg",
};

describe("build_media", () => {
  it("keeps the film fields it does not touch", () => {
    const new_media = build_media(film, { ...film_inputs, title: "Alien 1" });
    expect(new_media).toEqual({ ...film, title: "Alien 1" });
    expect(new_media.tmdb_id).toBe(348);
    expect(new_media.overview).toBe(film.overview);
  });

  it("does not change the media it was given", () => {
    const before = structuredClone(film);
    build_media(film, { ...film_inputs, title: "Other", countries: [] });
    expect(film).toEqual(before);
  });

  it("turns an unmatched file into a film", () => {
    expect(
      build_media(
        { kind: "unmatched" },
        { title: "T", original_title: "OT", countries: ["JP"], image: "/p.jpg" }
      )
    ).toEqual({
      kind: "film",
      title: "T",
      original_title: "OT",
      countries: ["JP"],
      poster_path: "/p.jpg",
    });
  });

  it("writes an episode's name and still, and leaves the rest alone", () => {
    const new_media = build_media(episode, {
      title: "New name",
      original_title: "ignored",
      countries: ["ignored"],
      image: "/new-still.jpg",
    });
    expect(new_media).toEqual({
      ...episode,
      name: "New name",
      still_path: "/new-still.jpg",
    });
    for (const film_field of [
      "title",
      "poster_path",
      "original_title",
      "countries",
    ]) {
      expect(new_media).not.toHaveProperty(film_field);
    }
  });

  it("saves empty text as null", () => {
    const new_media = build_media(film, {
      ...film_inputs,
      title: "",
      original_title: "",
      image: undefined,
    });
    expect(new_media.title).toBeNull();
    expect(new_media.original_title).toBeNull();
    expect(new_media.poster_path).toBeNull();
    expect(
      build_media(episode, { title: "", image: "/s.jpg" }).name
    ).toBeNull();
  });

  it("keeps an episode's details while show and numbers stay", () => {
    expect(
      build_media(episode, { ...episode_inputs, show_id: 3, season: 1, episode: 2 })
    ).toEqual(episode);
  });

  it("drops an episode's TMDB details once show or numbers change", () => {
    for (const change of [{ show_id: 4 }, { season: 2 }, { episode: null }]) {
      const new_media = build_media(episode, { ...episode_inputs, ...change });
      expect(new_media).toEqual({
        kind: "episode",
        show_id: 3,
        season: 1,
        episode: 2,
        ...change,
      });
    }
  });

  it("keeps a name or image the user entered on a moved episode", () => {
    expect(
      build_media(episode, { ...episode_inputs, title: "Mine", season: 2 })
    ).toEqual({ kind: "episode", show_id: 3, season: 2, episode: 2, name: "Mine" });
    expect(
      build_media(episode, { ...episode_inputs, image: "/mine.jpg", season: 2 })
        .still_path
    ).toBe("/mine.jpg");
  });

  it("drops film fields when a film becomes an episode", () => {
    const new_media = build_media(film, {
      ...film_inputs,
      kind: "episode",
      show_id: 3,
      season: 1,
      episode: 2,
    });
    expect(new_media).toEqual(episode_from_film);
  });

  it("drops episode fields when an episode becomes a film", () => {
    const new_media = build_media(episode, {
      ...episode_inputs,
      kind: "film",
      original_title: "",
    });
    expect(new_media).toEqual(film_from_episode);
  });

  it("gives an extra that becomes an episode only show and numbers", () => {
    expect(
      build_media(extra, { title: "", countries: [""], episode: 4 })
    ).toEqual(extra);
    expect(
      build_media(extra, { kind: "episode", title: "", countries: [""], episode: 4 })
    ).toEqual({ kind: "episode", show_id: 3, season: 1, episode: 4 });
  });

  it("drops empty country entries", () => {
    const countries = (input) =>
      build_media(film, { ...film_inputs, countries: input }).countries;
    expect(countries(["US", "", "GB", ""])).toEqual(["US", "GB"]);
    expect(countries([""])).toBeNull();
    expect(countries([])).toBeNull();
  });
});

describe("build_media output the backend has to accept", () => {
  it("film", () => {
    const before = { ...film, title: "Old title", poster_path: "/old.jpg" };
    expect(build_media(before, film_inputs)).toEqual(film);
  });

  it("user-titled film from an unmatched file", () => {
    expect(
      build_media(
        { kind: "unmatched" },
        { title: "Holiday 2019", original_title: "", countries: [""] }
      )
    ).toEqual(user_film);
  });

  it("episode", () => {
    const before = { ...episode, name: null, still_path: null };
    expect(
      build_media(before, {
        title: "Die Mitte der Welt",
        original_title: "",
        countries: [""],
        image: "/heimat-s01e02.jpg",
      })
    ).toEqual(episode);
  });

  it("episode moved to another show and season, named by the user", () => {
    expect(
      build_media(episode, {
        ...episode_inputs,
        title: "Christmas Special",
        show_id: 4,
        season: 0,
        episode: 5,
      })
    ).toEqual(episode_moved);
  });

  it("film switched to an episode", () => {
    expect(
      build_media(film, {
        ...film_inputs,
        kind: "episode",
        show_id: 3,
        season: 1,
        episode: 2,
      })
    ).toEqual(episode_from_film);
  });

  it("episode switched to an extra", () => {
    expect(
      build_media(episode, { ...episode_inputs, kind: "extra", show_id: 3, season: 1 })
    ).toEqual(extra);
  });

  it("episode switched to a film", () => {
    expect(
      build_media(episode, { ...episode_inputs, kind: "film", original_title: "" })
    ).toEqual(film_from_episode);
  });
});
