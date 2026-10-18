import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

import { ScaleWrapperGlobal } from "../UITools/ScaleWrapper.jsx";
import { ContextMenuProvider } from "../UITools/ContextMenu.jsx";
import { LibraryView } from "./LibraryView.jsx";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const FILM_PATH = "D:\\films\\alien.mkv";
const EPISODE_PATH = "D:\\tv\\heimat\\s01e02.mkv";
const FILM_TITLE = "Alien";
const SHOW_NAME = "Heimat";
const EPISODE_TITLE = "S01E02 · Die Mitte der Welt";

// The film is unwatched, the episode watched. The view changes elements in
// place, so every test gets its own.
const backend = () => ({
  get_library_name: "Test library",
  get_library_shows: {
    3: {
      name: SHOW_NAME,
      poster_path: "/heimat.jpg",
      seasons: [{ season_number: 1, name: "Season 1", episode_count: 11 }],
    },
  },
  get_library_videos: [
    {
      filepath: FILM_PATH,
      watched: false,
      length_in_seconds: 100,
      timestamp_modified: 2,
      media: { kind: "film", tmdb_id: 348, title: FILM_TITLE },
    },
    {
      filepath: EPISODE_PATH,
      watched: true,
      length_in_seconds: 50,
      timestamp_modified: 1,
      media: {
        kind: "episode",
        show_id: 3,
        season: 1,
        episode: 2,
        name: "Die Mitte der Welt",
      },
    },
  ],
  get_library_sort_preference: "title",
  get_library_filter_preferences: { watched_filter: "" },
  start_video_in_mpc: 4242,
  is_process_running: true,
});

// Tiles are found by their image, whose alt text is the shown title
const film_tile = () => screen.queryByAltText(FILM_TITLE);
const show_tile = () => screen.queryByAltText(SHOW_NAME);
const episode_tile = () => screen.queryByAltText(EPISODE_TITLE);

// An unwatched second episode of season 1 and one of season 2
const backend_with_more_episodes = () => {
  const responses = backend();
  const episode = responses.get_library_videos[1];
  responses.get_library_videos.push(
    {
      ...episode,
      filepath: "D:\tv\heimat\s01e01.mkv",
      watched: false,
      media: { ...episode.media, episode: 1, name: "Fernweh" },
    },
    {
      ...episode,
      filepath: "D:\tv\heimat\s02e01.mkv",
      watched: false,
      media: { ...episode.media, season: 2, episode: 1, name: null },
    }
  );
  return responses;
};

async function open_season_1() {
  fireEvent.click(show_tile());
  fireEvent.click(await screen.findByAltText("Season 1"));
  await screen.findByAltText(EPISODE_TITLE);
}

async function render_library() {
  render(
    <ScaleWrapperGlobal>
      <ContextMenuProvider>
        <MemoryRouter initialEntries={["/library/7"]}>
          <Routes>
            <Route path="/library/:library_id/*" element={<LibraryView />} />
          </Routes>
        </MemoryRouter>
      </ContextMenuProvider>
    </ScaleWrapperGlobal>
  );
  await screen.findByAltText(FILM_TITLE);
  // Let the sort and filter preferences arrive
  await act(async () => {});
}

beforeEach(() => {
  const responses = backend();
  invoke.mockReset().mockImplementation(async (command) => responses[command]);
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("LibraryView", () => {
  it("loads the show table and the videos and renders a film and a show", async () => {
    await render_library();

    expect(invoke).toHaveBeenCalledWith("get_library_shows", {
      library_id: "7",
    });
    expect(invoke).toHaveBeenCalledWith("get_library_videos", {
      library_id: "7",
    });
    expect(film_tile()).toBeTruthy();
    // The episode is inside its show's tile, which comes from the show table
    expect(episode_tile()).toBeNull();
    expect(show_tile().getAttribute("src")).toBe("/heimat.jpg");
    expect(screen.getByText("Watched 1/1")).toBeTruthy();
  });

  it("opens a show's seasons and a season's episodes", async () => {
    await render_library();

    await open_season_1();
    expect(screen.getByText("have 1 of 11, Watched 1/1")).toBeTruthy();
    // The still falls back to the show's poster
    expect(episode_tile().getAttribute("src")).toBe("/heimat.jpg");
  });

  it("marks a file watched from the context menu", async () => {
    await render_library();

    fireEvent.contextMenu(film_tile());
    fireEvent.click(screen.getByText("Mark watched"));

    expect(invoke).toHaveBeenCalledWith("set_watched", {
      library_id: "7",
      filepaths: [FILM_PATH],
      watched: true,
    });
    expect(screen.queryByText("Mark watched")).toBeNull();
  });

  it("marks a file unwatched from the context menu", async () => {
    await render_library();
    await open_season_1();

    fireEvent.contextMenu(episode_tile());
    fireEvent.click(screen.getByText("Mark unwatched"));

    expect(invoke).toHaveBeenCalledWith("set_watched", {
      library_id: "7",
      filepaths: [EPISODE_PATH],
      watched: false,
    });
  });

  it("filters by watched state", async () => {
    await render_library();
    expect(film_tile() && show_tile()).toBeTruthy();

    fireEvent.click(screen.getByText("Filter Watched"));
    expect(film_tile()).toBeTruthy();
    expect(show_tile()).toBeNull();

    fireEvent.click(screen.getByText("Filter Unwatched"));
    expect(film_tile()).toBeNull();
    expect(show_tile()).toBeTruthy();

    fireEvent.click(screen.getByText("Remove Filter"));
    expect(film_tile() && show_tile()).toBeTruthy();
  });

  it("counts a show as unwatched while any episode is", async () => {
    const responses = backend_with_more_episodes();
    invoke.mockImplementation(async (command) => responses[command]);
    await render_library();
    expect(screen.getByText("Watched 1/3")).toBeTruthy();

    fireEvent.click(screen.getByText("Filter Watched"));
    expect(show_tile()).toBeTruthy();
    fireEvent.click(screen.getByText("Filter Unwatched"));
    expect(show_tile()).toBeNull();
  });

  it("marks a whole show watched with one call, extras left alone", async () => {
    const responses = backend_with_more_episodes();
    const episode = responses.get_library_videos[1];
    responses.get_library_videos.push({
      ...episode,
      filepath: "D:\\tv\\heimat\\making of.mkv",
      watched: false,
      media: { kind: "episode", show_id: 3 },
    });
    invoke.mockImplementation(async (command) => responses[command]);
    await render_library();
    fireEvent.click(show_tile());
    fireEvent.click(await screen.findByText("Mark show watched"));

    const set_watched_calls = invoke.mock.calls.filter(
      ([command]) => command === "set_watched"
    );
    expect(set_watched_calls).toEqual([
      [
        "set_watched",
        {
          library_id: "7",
          filepaths: [
            EPISODE_PATH,
            "D:\tv\heimat\s01e01.mkv",
            "D:\tv\heimat\s02e01.mkv",
          ],
          watched: true,
        },
      ],
    ]);
    expect(screen.getByText("Watched 3/3, Extras: 0/1")).toBeTruthy();
    expect(screen.getByText("Mark show unwatched")).toBeTruthy();
  });

  it("marks a whole season watched with one call", async () => {
    const responses = backend_with_more_episodes();
    invoke.mockImplementation(async (command) => responses[command]);
    await render_library();
    await open_season_1();
    fireEvent.click(screen.getByText("Mark season watched"));

    const set_watched_calls = invoke.mock.calls.filter(
      ([command]) => command === "set_watched"
    );
    // Episode order, season 2 is left alone
    expect(set_watched_calls).toEqual([
      [
        "set_watched",
        {
          library_id: "7",
          filepaths: ["D:\tv\heimat\s01e01.mkv", EPISODE_PATH],
          watched: true,
        },
      ],
    ]);
  });

  it("marks a show or season watched from its tile's context menu, extras optional", async () => {
    const responses = backend_with_more_episodes();
    const episode = responses.get_library_videos[1];
    const EXTRA_PATH = "D:\\tv\\heimat\\s01 making of.mkv";
    responses.get_library_videos.push({
      ...episode,
      filepath: EXTRA_PATH,
      watched: false,
      media: { kind: "extra", show_id: 3, season: 1 },
    });
    invoke.mockImplementation(async (command) => responses[command]);
    await render_library();
    const set_watched_calls = () =>
      invoke.mock.calls
        .filter(([command]) => command === "set_watched")
        .map(([, args]) => args);

    fireEvent.contextMenu(show_tile());
    fireEvent.click(screen.getByText("Mark watched"));
    expect(set_watched_calls().pop().filepaths).not.toContain(EXTRA_PATH);
    expect(screen.getByText("Watched 3/3, Extras: 0/1")).toBeTruthy();

    fireEvent.click(show_tile());
    fireEvent.contextMenu(await screen.findByAltText("Season 1"));
    // Episodes are watched, extras are not
    expect(screen.getByText("Mark unwatched")).toBeTruthy();
    fireEvent.click(screen.getByText("Mark watched (with Extras)"));
    expect(set_watched_calls().pop()).toEqual({
      library_id: "7",
      filepaths: ["D:\tv\heimat\s01e01.mkv", EPISODE_PATH, EXTRA_PATH],
      watched: true,
    });
  });

  describe("editing", () => {
    const calls_of = (command) =>
      invoke.mock.calls.filter(([called]) => called === command);
    const change = (selector, value) =>
      fireEvent.change(document.querySelector(selector), {
        target: { value },
      });
    async function search_tv(text) {
      change("#searchinput", text);
      await act(async () => {
        fireEvent.keyDown(document.querySelector("#searchinput"), {
          key: "Enter",
        });
      });
    }
    const with_responses = (extra) => {
      const responses = { ...backend(), ...extra };
      invoke.mockImplementation(async (command) => responses[command]);
    };

    it("files a film under a new show from TMDB through set_media, then reloads", async () => {
      with_responses({
        search_tv_from_gui: [{ tmdb_id: 1400, name: "Seinfeld" }],
        add_show_from_tmdb: 9,
      });
      await render_library();
      fireEvent.contextMenu(film_tile());
      fireEvent.click(screen.getByText("Edit"));

      change("#kind_input", "episode");
      await search_tv("Seinfeld");
      fireEvent.click(screen.getByAltText("Seinfeld"));
      change("#season_input", "1");
      change("#episode_input", "2");
      await act(async () => {
        fireEvent.click(screen.getByText("Save Changes"));
      });

      expect(calls_of("add_show_from_tmdb")).toEqual([
        ["add_show_from_tmdb", { library_id: "7", tmdb_id: 1400 }],
      ]);
      expect(calls_of("set_media")).toEqual([
        [
          "set_media",
          {
            library_id: "7",
            filepath: FILM_PATH,
            media: { kind: "episode", show_id: 9, season: 1, episode: 2 },
          },
        ],
      ]);
      // The backend looked the episode up and may have moved it to another tile
      expect(calls_of("get_library_videos")).toHaveLength(2);
      expect(calls_of("get_library_shows")).toHaveLength(2);
    });

    it("edits a show's name from its tile's context menu", async () => {
      await render_library();
      fireEvent.contextMenu(show_tile());
      fireEvent.click(screen.getByText("Edit show"));

      change("#show_name_input", "Heimat – Eine deutsche Chronik");
      await act(async () => {
        fireEvent.click(screen.getByText("Save Changes"));
      });

      expect(calls_of("update_show")).toEqual([
        [
          "update_show",
          {
            library_id: "7",
            show_id: 3,
            name: "Heimat – Eine deutsche Chronik",
            poster_path: "/heimat.jpg",
          },
        ],
      ]);
      expect(calls_of("get_library_shows")).toHaveLength(2);
    });

    it("rematches a show from the show view", async () => {
      with_responses({
        search_tv_from_gui: [
          { tmdb_id: 4613, name: "Heimat", first_air_date: "1984-09-16" },
        ],
        rematch_show: 3,
      });
      await render_library();
      fireEvent.click(show_tile());
      fireEvent.click(await screen.findByText("Edit show"));

      await search_tv("Heimat");
      fireEvent.click(screen.getByAltText("Heimat"));
      await act(async () => {
        fireEvent.click(screen.getByText("Rematch to Heimat (1984)"));
      });

      expect(calls_of("rematch_show")).toEqual([
        ["rematch_show", { library_id: "7", show_id: 3, tmdb_id: 4613 }],
      ]);
      expect(calls_of("get_library_videos")).toHaveLength(2);
      // Back on the show
      expect(await screen.findByText("Edit show")).toBeTruthy();
    });
  });

  it("starts with the saved watched filter", async () => {
    const responses = {
      ...backend(),
      get_library_filter_preferences: { watched_filter: "filter_watched" },
    };
    invoke.mockImplementation(async (command) => responses[command]);
    await render_library();

    expect(film_tile()).toBeTruthy();
    expect(episode_tile()).toBeNull();
  });

  it("marks a video watched once it ran 80% with the player still open", async () => {
    await render_library();
    const set_watched_calls = () =>
      invoke.mock.calls.filter(([command]) => command === "set_watched");

    // Only setTimeout: React's act needs its own scheduling left alone
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    fireEvent.click(film_tile());
    await act(() => vi.advanceTimersByTimeAsync(0));
    expect(invoke).toHaveBeenCalledWith(
      "start_video_in_mpc",
      expect.objectContaining({ filepath: FILM_PATH })
    );

    // The film is 100 seconds long
    await act(() => vi.advanceTimersByTimeAsync(79_999));
    expect(set_watched_calls()).toEqual([]);

    await act(() => vi.advanceTimersByTimeAsync(1));
    expect(invoke).toHaveBeenCalledWith("is_process_running", {
      process_id: 4242,
    });
    expect(set_watched_calls()).toEqual([
      [
        "set_watched",
        { library_id: "7", filepaths: [FILM_PATH], watched: true },
      ],
    ]);
  });

  it("leaves a video unwatched when the player was closed before 80%", async () => {
    const responses = { ...backend(), is_process_running: false };
    invoke.mockImplementation(async (command) => responses[command]);
    await render_library();

    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    fireEvent.click(film_tile());
    await act(() => vi.advanceTimersByTimeAsync(0));
    await act(() => vi.advanceTimersByTimeAsync(80_000));

    expect(invoke).toHaveBeenCalledWith("is_process_running", {
      process_id: 4242,
    });
    expect(invoke).not.toHaveBeenCalledWith("set_watched", expect.anything());
  });
});
