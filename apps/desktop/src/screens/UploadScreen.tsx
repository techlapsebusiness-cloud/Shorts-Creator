import { useState } from "react";
import { useSession } from "../session";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Toggle } from "../components/Toggle";
import {
  STUDIO_URL,
  TITLE_LIMIT,
  UNVERIFIED_NOTICE,
  buildKit,
  directUploadAllowed,
  plainText,
} from "../upload/kit";
import styles from "./screens.module.css";

export function UploadScreen() {
  const session = useSession();
  const hook =
    session.project?.ai.beats?.[0]?.text || "This file survives a crash";
  const [keyword, setKeyword] = useState(session.project?.ai.topic || "rename");
  const [direct, setDirect] = useState(false);
  const [note, setNote] = useState("Copy a field, then paste it into Studio.");
  const kit = buildKit(keyword, hook, "https://example.com/long", "exposure");
  const allowed = directUploadAllowed(direct, "");

  async function copy(label: string, text: string) {
    try {
      await navigator.clipboard.writeText(text);
      setNote(`Copied ${label}.`);
    } catch {
      setNote(`Could not copy ${label}.`);
    }
  }

  return (
    <div className={`${styles.page} ${styles.split}`}>
      <Card title="Upload kit">
        <p className={styles.kicker}>Keyword</p>
        <input
          aria-label="Keyword"
          value={keyword}
          onChange={(event) => setKeyword(event.target.value)}
        />
        <p className={styles.kicker}>Title</p>
        <p>
          {kit.title}{" "}
          <span className={styles.meta}>
            {kit.title.length}/{TITLE_LIMIT}
          </span>
        </p>
        <Button onClick={() => copy("title", kit.title)}>Copy title</Button>
        <p className={styles.kicker}>Description</p>
        <p>{kit.description}</p>
        <Button onClick={() => copy("description", kit.description)}>
          Copy description
        </Button>
        <p className={styles.kicker}>Tags</p>
        <p className={styles.meta}>{kit.tags}</p>
        <Button onClick={() => copy("tags", kit.tags)}>Copy tags</Button>
        <p className={styles.kicker}>Pinned comment</p>
        <p>{kit.pinnedComment}</p>
        <Button onClick={() => copy("pinned comment", kit.pinnedComment)}>
          Copy comment
        </Button>
        <Button tone="primary" onClick={() => copy("the kit", plainText(kit))}>
          Copy all
        </Button>
        <p className={styles.meta}>{note}</p>
      </Card>
      <div className={styles.stack}>
        <Card title="Studio settings">
          <ul className={styles.checks}>
            {kit.studio.map((field) => (
              <li key={field.label}>
                {field.label}
                <span className={styles.meta}>{field.value}</span>
              </li>
            ))}
          </ul>
          <p className={styles.meta}>Policy pack {kit.checkedOn}</p>
        </Card>
        <Card title="Files">
          <p className={styles.meta}>
            {session.dir
              ? `${session.dir}/renders/short.mp4`
              : "renders/short.mp4"}
          </p>
          <p className={styles.meta}>
            {session.dir
              ? `${session.dir}/renders/cover.png`
              : "renders/cover.png"}
          </p>
          <p className={styles.meta}>
            {session.dir
              ? `${session.dir}/renders/captions.srt`
              : "renders/captions.srt"}
          </p>
          <p className={styles.meta}>
            {session.dir
              ? `${session.dir}/renders/upload_kit.txt`
              : "renders/upload_kit.txt"}
          </p>
          <p className={styles.meta}>
            Originality · title saved for the last 10
          </p>
          <p className={styles.meta}>
            Check the public cut again after 24-48 h.
          </p>
          <Button onClick={() => window.open(STUDIO_URL, "_blank", "noopener")}>
            Open YouTube Studio
          </Button>
          <Toggle label="Direct upload" checked={direct} onChange={setDirect} />
          {direct ? <p className={styles.meta}>{UNVERIFIED_NOTICE}</p> : null}
          <p className={styles.meta}>
            {allowed
              ? "Upload would call videos.insert."
              : "Direct upload stays off until you add an OAuth client."}
          </p>
        </Card>
      </div>
    </div>
  );
}
