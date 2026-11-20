import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Toggle } from "../components/Toggle";
import styles from "./screens.module.css";

const SETTINGS = [
  ["Audience", "Not made for kids"],
  ["Altered content", "No"],
  ["Category", "Science & Technology"],
  ["Video language", "Hindi"],
  ["Related video", "None"],
  ["Custom thumbnail", "cover.png"],
  ["Comments", "On"],
  ["Pinned question", "What should the next crash test be?"],
  ["Remixing", "Off"],
  ["Visibility", "Private"],
  ["Length", "0:24"],
] as const;

export function UploadScreen() {
  return (
    <div className={`${styles.page} ${styles.split}`}>
      <Card title="Upload kit">
        <p className={styles.kicker}>Title</p>
        <p>This file survives a crash</p>
        <Button>Copy title</Button>
        <p className={styles.kicker}>Description</p>
        <p>The temp file never replaces the last good save. #Shorts</p>
        <Button>Copy description</Button>
        <p className={styles.kicker}>Tags</p>
        <p className={styles.meta}>shorts, ffmpeg, project json</p>
        <Button>Copy tags</Button>
        <Button tone="primary">Copy all</Button>
      </Card>
      <div className={styles.stack}>
        <Card title="Studio settings">
          <ul className={styles.checks}>
            {SETTINGS.map(([name, value]) => (
              <li key={name}>
                {name}
                <span className={styles.meta}>{value}</span>
              </li>
            ))}
          </ul>
        </Card>
        <Card title="Files">
          <p className={styles.meta}>renders/example-short.mp4</p>
          <p className={styles.meta}>renders/cover.png</p>
          <p className={styles.meta}>renders/captions.srt</p>
          <p className={styles.meta}>Originality · no earlier title matches</p>
          <p className={styles.meta}>
            Check the public cut again after 24-48 h.
          </p>
          <Button>Open YouTube Studio</Button>
          <Toggle label="Direct upload" checked={false} />
        </Card>
      </div>
    </div>
  );
}
