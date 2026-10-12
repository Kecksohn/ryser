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
const EPISODE_TITLE = "Heimat S01E02 Die Mitte der Welt";

// The film is unwatched, the episode watched. The view changes elements in
// place, so every test gets its own.
const backend = () => ({
  get_library_name: "Test library",
  get_library_shows: { 3: { name: "Heimat", poster_path: "/heimat.jpg" } },
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
const episode_tile = () => screen.queryByAltText(EPISODE_TITLE);

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
  it("loads the show table and the videos and renders both kinds", async () => {
    await render_library();

    expect(invoke).toHaveBeenCalledWith("get_library_shows", {
      library_id: "7",
    });
    expect(invoke).toHaveBeenCalledWith("get_library_videos", {
      library_id: "7",
    });
    expect(film_tile()).toBeTruthy();
    // Composed from the show table; the still falls back to the show's poster
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
    expect(film_tile() && episode_tile()).toBeTruthy();

    fireEvent.click(screen.getByText("Filter Watched"));
    expect(film_tile()).toBeTruthy();
    expect(episode_tile()).toBeNull();

    fireEvent.click(screen.getByText("Filter Unwatched"));
    expect(film_tile()).toBeNull();
    expect(episode_tile()).toBeTruthy();

    fireEvent.click(screen.getByText("Remove Filter"));
    expect(film_tile() && episode_tile()).toBeTruthy();
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
