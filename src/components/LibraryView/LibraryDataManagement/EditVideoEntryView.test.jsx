import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { EditVideoEntryView } from "./EditVideoEntryView.jsx";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const shows = { 3: { name: "Heimat", tmdb_id: 4613 } };

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
    });
  });

  it("shows the show's name instead of the film inputs for an episode", () => {
    const { container } = render_view(episode_entry());

    expect(screen.getByText("Show: Heimat")).toBeTruthy();
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
