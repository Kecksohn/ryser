import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { EditVideoEntryView } from "./EditVideoEntryView.jsx";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const shows = {
  3: { id: 3, name: "Heimat", tmdb_id: 4613 },
  4: { id: 4, name: "Braunschlag", tmdb_id: 43879 },
};

const film_entry = () => ({
  filepath: "D:\\films\\alien.mkv",
  watched: false,
  media: {
    kind: "film",
    tmdb_id: 348,
    title: "Alien",
    original_title: "Alien",
    overview: "A commercial crew is woken by a distress call.",
    countries: ["US"],
    poster_path: "/alien.jpg",
  },
});

const episode_entry = () => ({
  filepath: "D:\\tv\\heimat\\s01e02.mkv",
  watched: false,
  media: {
    kind: "episode",
    show_id: 3,
    season: 1,
    episode: 2,
    tmdb_id: 9001,
    name: "Die Mitte der Welt",
    still_path: "/still.jpg",
  },
});

function render_view(video_entry) {
  const save_media = vi.fn();
  const { container } = render(
    <MemoryRouter>
      <EditVideoEntryView
        save_media={save_media}
        video_entry={video_entry}
        shows={shows}
      />
    </MemoryRouter>
  );
  return { container, save_media };
}

function change(container, selector, value) {
  fireEvent.change(container.querySelector(selector), { target: { value } });
}

async function search(container, text) {
  change(container, "#searchinput", text);
  await act(async () => {
    fireEvent.keyDown(container.querySelector("#searchinput"), { key: "Enter" });
  });
}

// The cover is the first image as long as no TMDB results are listed
async function hover_cover(container) {
  await act(async () => {
    fireEvent.mouseEnter(container.querySelector("img"));
  });
}

beforeEach(() => {
  invoke.mockReset().mockResolvedValue([]);
});
afterEach(cleanup);

describe("EditVideoEntryView", () => {
  it("saves a film with the edited inputs", () => {
    const entry = film_entry();
    const { container, save_media } = render_view(entry);

    fireEvent.change(container.querySelector("#en_title_input"), {
      target: { value: "Alien 1" },
    });
    fireEvent.change(container.querySelector("#country_input"), {
      target: { value: "US, GB, " },
    });
    fireEvent.click(screen.getByText("Save Changes"));

    expect(save_media).toHaveBeenCalledTimes(1);
    expect(save_media).toHaveBeenCalledWith(entry, {
      ...film_entry().media,
      title: "Alien 1",
      countries: ["US", "GB"],
    }, undefined);
  });

  it("offers to save only while an input differs from the entry", () => {
    for (const entry of [
      film_entry(),
      episode_entry(),
      { ...film_entry(), media: { kind: "unmatched" } },
    ]) {
      const { container } = render_view(entry);
      const title_input = container.querySelector("#en_title_input");
      const before = title_input.value;
      expect(screen.queryByText("Save Changes")).toBeNull();

      fireEvent.change(title_input, { target: { value: "Other" } });
      expect(screen.getByText("Save Changes")).toBeTruthy();
      expect(screen.getByText("Discard Changes")).toBeTruthy();

      fireEvent.change(title_input, { target: { value: before } });
      expect(screen.queryByText("Save Changes")).toBeNull();
      cleanup();
    }
  });

  it("does not count an empty country entry as a change", () => {
    const { container } = render_view(film_entry());
    fireEvent.change(container.querySelector("#country_input"), {
      target: { value: "US, " },
    });
    expect(screen.queryByText("Save Changes")).toBeNull();
  });

  it("shows the show's name instead of the film inputs for an episode", () => {
    const { container } = render_view(episode_entry());

    expect(container.querySelector("#show_input").value).toBe("3");
    expect(container.querySelector("#season_input").value).toBe("1");
    expect(container.querySelector("#episode_input").value).toBe("2");
    expect(container.querySelector("#og_title_input")).toBeNull();
    expect(container.querySelector("#country_input")).toBeNull();
    expect(container.querySelector("#en_title_input").value).toBe(
      "Die Mitte der Welt"
    );
  });

  it("saves an episode's name without film fields", () => {
    const entry = episode_entry();
    const { container, save_media } = render_view(entry);

    fireEvent.change(container.querySelector("#en_title_input"), {
      target: { value: "Fernweh" },
    });
    fireEvent.click(screen.getByText("Save Changes"));

    expect(save_media).toHaveBeenCalledWith(entry, {
      ...episode_entry().media,
      name: "Fernweh",
    }, undefined);
  });

  it("saves show, season and episode; details of the old episode are dropped", () => {
    const entry = episode_entry();
    const { container, save_media } = render_view(entry);
    change(container, "#show_input", "4");
    change(container, "#season_input", "0");
    change(container, "#episode_input", "");
    fireEvent.click(screen.getByText("Save Changes"));

    expect(save_media).toHaveBeenCalledWith(
      entry,
      { kind: "episode", show_id: 4, season: 0, episode: null },
      undefined
    );
  });

  it("switches a film to an episode of a library show", () => {
    const entry = film_entry();
    const { container, save_media } = render_view(entry);
    change(container, "#kind_input", "episode");
    expect(container.querySelector("#country_input")).toBeNull();
    // An episode needs a show
    expect(screen.queryByText("Save Changes")).toBeNull();

    change(container, "#show_input", "3");
    change(container, "#season_input", "1");
    change(container, "#episode_input", "2");
    fireEvent.click(screen.getByText("Save Changes"));

    expect(save_media).toHaveBeenCalledWith(
      entry,
      { kind: "episode", show_id: 3, season: 1, episode: 2 },
      undefined
    );
  });

  it("switches an episode to an extra, which has only show and season", () => {
    const entry = episode_entry();
    const { container, save_media } = render_view(entry);
    change(container, "#kind_input", "extra");
    expect(container.querySelector("#episode_input")).toBeNull();
    expect(container.querySelector("#en_title_input")).toBeNull();
    change(container, "#season_input", "2");
    fireEvent.click(screen.getByText("Save Changes"));

    expect(save_media).toHaveBeenCalledWith(
      entry,
      { kind: "extra", show_id: 3, season: 2 },
      undefined
    );
  });

  it("opens an extra as an extra", () => {
    const entry = {
      ...episode_entry(),
      media: { kind: "extra", show_id: 4, season: null },
    };
    const { container } = render_view(entry);
    expect(container.querySelector("#kind_input").value).toBe("extra");
    expect(container.querySelector("#show_input").value).toBe("4");
    expect(screen.queryByText("Save Changes")).toBeNull();
  });

  it("switches an episode to a film", () => {
    const entry = episode_entry();
    const { container, save_media } = render_view(entry);
    change(container, "#kind_input", "film");
    change(container, "#og_title_input", "Heimat");
    fireEvent.click(screen.getByText("Save Changes"));

    expect(save_media).toHaveBeenCalledWith(
      entry,
      {
        kind: "film",
        title: "Die Mitte der Welt",
        original_title: "Heimat",
        countries: null,
        poster_path: "/still.jpg",
      },
      undefined
    );
  });

  it("picks the show of an episode from a TV search", async () => {
    invoke.mockImplementation(async (command) =>
      command === "search_tv_from_gui"
        ? [
            { tmdb_id: 1400, name: "Seinfeld", poster_path: "/seinfeld.jpg" },
            { tmdb_id: 43879, name: "Braunschlag", poster_path: "/b.jpg" },
          ]
        : []
    );
    const entry = episode_entry();
    const { container, save_media } = render_view(entry);
    await search(container, "Seinfeld");
    expect(invoke).toHaveBeenCalledWith("search_tv_from_gui", {
      search_title: "Seinfeld",
    });

    // A show the library has is picked from the library
    fireEvent.click(screen.getByAltText("Braunschlag"));
    expect(container.querySelector("#show_input").value).toBe("4");

    // A new one is added on save
    fireEvent.click(screen.getByAltText("Seinfeld"));
    expect(container.querySelector("#show_input").value).toBe("new");
    fireEvent.click(screen.getByText("Save Changes"));
    expect(save_media).toHaveBeenCalledWith(
      entry,
      { kind: "episode", show_id: null, season: 1, episode: 2 },
      1400
    );
  });

  it("searches films for a film", async () => {
    const { container } = render_view(film_entry());
    await search(container, "Alien");
    expect(invoke).toHaveBeenCalledWith("search_tmdb_from_gui", {
      search_title: "Alien",
    });
  });

  it("asks for a film's covers with the film's tmdb_id", async () => {
    const { container } = render_view(film_entry());
    await hover_cover(container);

    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("get_covers_from_tmdb", {
      tmdb_id: 348,
      is_tv: false,
    });
  });

  it("asks for an episode's covers with its show's tmdb_id", async () => {
    const { container } = render_view(episode_entry());
    await hover_cover(container);

    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("get_covers_from_tmdb", {
      tmdb_id: 4613,
      is_tv: true,
    });
  });

  it("does not ask for covers without a tmdb_id", async () => {
    const user_film = film_entry();
    user_film.media.tmdb_id = null;
    await hover_cover(render_view(user_film).container);

    const unmatched = { ...film_entry(), media: { kind: "unmatched" } };
    await hover_cover(render_view(unmatched).container);

    // The show was not found on TMDB; the episode's own tmdb_id must not be used
    const episode = episode_entry();
    episode.media.show_id = 99;
    await hover_cover(render_view(episode).container);

    expect(invoke).not.toHaveBeenCalled();
  });
});
