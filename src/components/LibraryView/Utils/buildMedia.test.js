import { describe, expect, it } from "vitest";

import { build_media } from "./buildMedia.js";

// Also deserialized by the Rust tests in video_element.rs
import film from "../../../../test-fixtures/media/film.json";
import user_film from "../../../../test-fixtures/media/user_film.json";
import episode from "../../../../test-fixtures/media/episode.json";

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
});
