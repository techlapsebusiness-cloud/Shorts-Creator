import { invoke } from "@tauri-apps/api/core";
import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import type { Project } from "./bindings/Project";

export type OpenedTake = {
  project: Project;
  dir: string;
};

type SessionValue = {
  project: Project | null;
  dir: string | null;
  setOpened: (opened: OpenedTake) => void;
  updateProject: (project: Project) => void;
};

const SessionContext = createContext<SessionValue>({
  project: null,
  dir: null,
  setOpened: () => undefined,
  updateProject: () => undefined,
});

export function SessionProvider({ children }: { children: ReactNode }) {
  const [project, setProject] = useState<Project | null>(null);
  const [dir, setDir] = useState<string | null>(null);

  useEffect(() => {
    if (!project || !dir) {
      return;
    }
    const handle = window.setTimeout(() => {
      void invoke("save_opened_project", { dir, project }).catch(
        () => undefined,
      );
    }, 5000);
    return () => window.clearTimeout(handle);
  }, [project, dir]);

  return (
    <SessionContext.Provider
      value={{
        project,
        dir,
        setOpened: (opened) => {
          setProject(hydrate(opened.project));
          setDir(opened.dir);
        },
        updateProject: (next) => setProject(hydrate(next)),
      }}
    >
      {children}
    </SessionContext.Provider>
  );
}

export function useSession() {
  return useContext(SessionContext);
}

export function hydrate(project: Project): Project {
  return {
    ...project,
    layout: {
      ...project.layout,
      transition: project.layout.transition || "cut",
      markersSecs: project.layout.markersSecs ?? [],
    },
    tracks: project.tracks.map((track) => ({
      ...track,
      muted: track.muted ?? false,
      solo: track.solo ?? false,
    })),
  };
}
