export type RouteId =
  "record" | "edit" | "audio" | "assets" | "export" | "upload" | "settings";

export type Route = {
  id: RouteId;
  path: string;
  label: string;
  action: string;
};

export const STEPS: Route[] = [
  {
    id: "record",
    path: "/",
    label: "01 Record",
    action: "Start recording",
  },
  { id: "edit", path: "/edit", label: "02 Edit", action: "Split clip" },
  { id: "audio", path: "/audio", label: "03 Audio", action: "Remove cuts" },
  {
    id: "assets",
    path: "/assets",
    label: "04 Auto assets",
    action: "Add to new Shorts",
  },
  { id: "export", path: "/export", label: "05 Export", action: "Render" },
  {
    id: "upload",
    path: "/upload",
    label: "06 Upload",
    action: "Open YouTube Studio",
  },
];

export const SETTINGS: Route = {
  id: "settings",
  path: "/settings",
  label: "AI settings",
  action: "Save settings",
};

const ALL = [...STEPS, SETTINGS];

export function routeFromPath(path: string): Route {
  const clean =
    path.length > 1 && path.endsWith("/") ? path.slice(0, -1) : path;
  return ALL.find((route) => route.path === clean) ?? STEPS[0];
}
