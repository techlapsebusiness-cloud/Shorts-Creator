export const TITLE_LIMIT = 100;
export const POLICY_CHECKED_ON = "2026-11-22";
export const UNVERIFIED_NOTICE =
  "An unverified Google API project can only upload as private until Google audits it.";
export const STUDIO_URL = "https://studio.youtube.com";

const BANNED = ["viral", "fyp"];

export type StudioField = { label: string; value: string };

export const STUDIO: StudioField[] = [
  { label: "Audience", value: "Not made for kids" },
  { label: "Altered content", value: "No" },
  { label: "Category", value: "Science & Technology" },
  { label: "Video language", value: "Hindi" },
  { label: "Related video", value: "None" },
  { label: "Custom thumbnail", value: "cover.png" },
  { label: "Comments", value: "On" },
  { label: "Remixing", value: "Off" },
  { label: "Visibility", value: "Private" },
];

export type UploadKit = {
  title: string;
  description: string;
  tags: string;
  pinnedComment: string;
  studio: StudioField[];
  checkedOn: string;
};

export function keywordTitle(
  keyword: string,
  hook: string,
  limit = TITLE_LIMIT,
): string {
  const key = keyword.trim();
  const line = hook.trim();
  const combined = line.toLowerCase().startsWith(key.toLowerCase())
    ? line
    : `${key}: ${line}`;
  return [...combined].slice(0, limit).join("");
}

export function kitDescription(
  hook: string,
  longUrl: string,
  extra: string[],
): string {
  const tags = ["#Shorts"];
  for (const tag of extra) {
    const bare = tag.trim().replace(/^#/, "");
    if (!bare || BANNED.includes(bare.toLowerCase())) {
      continue;
    }
    const hashed = `#${bare}`;
    if (tags.some((item) => item.toLowerCase() === hashed.toLowerCase())) {
      continue;
    }
    if (tags.length === 3) {
      break;
    }
    tags.push(hashed);
  }
  return `${hook.trim()}\n${longUrl.trim()}\n${tags.join(" ")}`;
}

export function buildKit(
  keyword: string,
  hook: string,
  longUrl: string,
  nextTopic: string,
): UploadKit {
  return {
    title: keywordTitle(keyword, hook),
    description: kitDescription(hook, longUrl, ["#howto"]),
    tags: [keyword.trim(), "shorts", "ffmpeg"]
      .map((tag) => tag.trim().replace(/^#/, ""))
      .filter((tag, index, all) => {
        if (!tag || BANNED.includes(tag.toLowerCase())) {
          return false;
        }
        return (
          all.findIndex((item) => item.toLowerCase() === tag.toLowerCase()) ===
          index
        );
      })
      .join(", "),
    pinnedComment: `What should the next short about ${nextTopic} cover?`,
    studio: STUDIO,
    checkedOn: POLICY_CHECKED_ON,
  };
}

export function plainText(kit: UploadKit): string {
  const fields = kit.studio
    .map((field) => `${field.label}: ${field.value}`)
    .join("\n");
  return `Title\n${kit.title}\n\nDescription\n${kit.description}\n\nTags\n${kit.tags}\n\nPinned comment\n${kit.pinnedComment}\n\n${fields}\n`;
}

export function directUploadAllowed(
  toggle: boolean,
  clientId: string,
): boolean {
  return toggle && clientId.trim().length > 0;
}
