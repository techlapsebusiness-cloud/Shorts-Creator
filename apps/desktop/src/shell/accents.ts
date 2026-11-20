export const ACCENTS = [
  { id: "rust", label: "Rust", value: "#FF7A3D" },
  { id: "mint", label: "Mint", value: "#4FE3C1" },
  { id: "amber", label: "Amber", value: "#F5B26B" },
  { id: "blue", label: "Blue", value: "#8AB4FF" },
] as const;

export type AccentId = (typeof ACCENTS)[number]["id"];

export function applyAccent(id: AccentId): void {
  const accent = ACCENTS.find((item) => item.id === id);
  if (!accent) {
    return;
  }
  document.documentElement.style.setProperty("--accent", accent.value);
}
