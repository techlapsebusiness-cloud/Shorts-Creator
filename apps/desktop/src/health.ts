export type Health = {
  name: string;
  version: string;
};

export function readHealthName(value: unknown): string | null {
  if (typeof value !== "object" || value === null) {
    return null;
  }
  if (
    !("name" in value) ||
    typeof value.name !== "string" ||
    value.name.length === 0
  ) {
    return null;
  }
  if (
    !("version" in value) ||
    typeof value.version !== "string" ||
    value.version.length === 0
  ) {
    return null;
  }
  return value.name;
}
