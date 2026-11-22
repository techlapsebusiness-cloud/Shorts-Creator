import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Toggle } from "../components/Toggle";
import {
  POLICY_CHECKED_ON,
  blocks,
  exampleShot,
  preflight,
} from "../export/preflight";
import { presetById, sizeLabel } from "../export/settings";
import { useSession } from "../session";
import styles from "./screens.module.css";

type RenderStatus = {
  phase: string;
  framesDone: number;
  framesTotal: number;
  etaSecs: number;
  output: string | null;
  encoder: string | null;
  message: string | null;
};

export function ExportScreen() {
  const session = useSession();
  const [resolution, setResolution] = useState("1080p");
  const [fps, setFps] = useState(30);
  const [ease, setEase] = useState(true);
  const [sixty, setSixty] = useState(true);
  const [blur, setBlur] = useState(false);
  const [cursor, setCursor] = useState(true);
  const [blend, setBlend] = useState(true);
  const [cover, setCover] = useState(true);
  const [srt, setSrt] = useState(true);
  const [meta, setMeta] = useState(true);
  const [licensed, setLicensed] = useState(true);
  const [status, setStatus] = useState<RenderStatus | null>(null);
  const preset = presetById(resolution);
  const rendering = status?.phase === "rendering";

  useEffect(() => {
    if (!rendering) {
      return;
    }
    let cancel = false;
    const timer = window.setInterval(() => {
      invoke<RenderStatus>("render_status")
        .then((next) => {
          if (!cancel) {
            setStatus(next);
          }
        })
        .catch(() => undefined);
    }, 400);
    return () => {
      cancel = true;
      window.clearInterval(timer);
    };
  }, [rendering]);

  return (
    <div className={styles.page}>
      <Card title="Resolution">
        <div className={styles.cards}>
          {["4K", "1440p", "1080p"].map((name) => {
            const item = presetById(name);
            return (
              <button
                key={name}
                type="button"
                className={styles.choice}
                aria-pressed={resolution === name}
                onClick={() => setResolution(name)}
              >
                <strong>{name}</strong>
                <span className={styles.meta}>
                  {item.width} x {item.height}
                </span>
              </button>
            );
          })}
        </div>
        <div className={styles.row}>
          {[30, 60].map((rate) => (
            <Button
              key={rate}
              tone={fps === rate ? "primary" : "quiet"}
              onClick={() => setFps(rate)}
            >
              {rate} fps
            </Button>
          ))}
        </div>
        <p className={styles.meta}>
          {fps} fps · h264 · {preset.bitrateKbps} kbps video ·{" "}
          {sizeLabel(preset.bitrateKbps, 60)} for 60 s
        </p>
      </Card>
      <div className={styles.split}>
        <Card title="Motion smoothness">
          <Toggle label="Ease in out" checked={ease} onChange={setEase} />
          <Toggle
            label="Animate at 60 fps"
            checked={sixty}
            onChange={setSixty}
          />
          <Toggle
            label="Motion blur on fast moves"
            checked={blur}
            onChange={setBlur}
          />
          <Toggle
            label="Smooth cursor path"
            checked={cursor}
            onChange={setCursor}
          />
          <Toggle
            label="Frame blending on speed-ups"
            checked={blend}
            onChange={setBlend}
          />
        </Card>
        <Card title="Also export">
          <Toggle label="Cover PNG" checked={cover} onChange={setCover} />
          <Toggle label="Captions .srt" checked={srt} onChange={setSrt} />
          <Toggle label="Metadata .txt" checked={meta} onChange={setMeta} />
        </Card>
      </div>
      <Card title="Pre-flight">
        <ul className={styles.checks}>
          {preflight({
            ...exampleShot(licensed),
            width: preset.width,
            height: preset.height,
            durationSecs:
              session.project?.tracks
                .find((track) => track.id === "V1")
                ?.clips.reduce(
                  (end, clip) =>
                    Math.max(end, clip.timelineStart + clip.duration),
                  0,
                ) || exampleShot(licensed).durationSecs,
            safeZone: session.project?.layout.safeZone ?? true,
            topic: session.project?.ai.topic || exampleShot(licensed).topic,
            title: session.project?.name || exampleShot(licensed).title,
          }).map((check) => (
            <li key={check.id}>
              <span>
                {check.label}
                <span className={styles.meta}> {check.detail}</span>
              </span>
              <span
                className={
                  check.state === "pass"
                    ? styles.pass
                    : check.state === "warn"
                      ? styles.warn
                      : styles.fail
                }
              >
                {check.state === "pass"
                  ? "Pass"
                  : check.state === "warn"
                    ? "Warn"
                    : "Fail"}
              </span>
            </li>
          ))}
        </ul>
        <Toggle
          label="Music licence confirmed"
          checked={licensed}
          onChange={setLicensed}
        />
        <p className={styles.meta}>
          Policy pack {POLICY_CHECKED_ON}.{" "}
          {blocks() ? "Export is blocked." : "Warnings do not block export."}
        </p>
        <p className={styles.meta}>
          {status?.output ?? "renders/"} · {status?.phase ?? "idle"}
          {status?.encoder ? ` · ${status.encoder}` : ""}
          {rendering ? ` · ${Math.round(status?.etaSecs ?? 0)}s left` : ""}
        </p>
        <div className={styles.queue}>
          <span
            style={{
              width: `${status && status.framesTotal > 0 ? (status.framesDone / status.framesTotal) * 100 : 0}%`,
            }}
          />
        </div>
        <div className={styles.row}>
          <Button
            tone="primary"
            disabled={rendering}
            onClick={() => {
              const next = {
                phase: "rendering",
                framesDone: 0,
                framesTotal: session.project
                  ? 30
                  : preset.width >= 1440
                    ? 4
                    : 8,
                etaSecs: 0,
                output: session.dir ? `${session.dir}/renders/short.mp4` : null,
                encoder: null,
                message: null,
              };
              setStatus(next);
              invoke("start_render", {
                args: {
                  width: preset.width,
                  height: preset.height,
                  fps,
                  projectDir: session.dir,
                  cover,
                  captions: srt,
                },
              }).catch((error: unknown) => {
                setStatus({
                  ...next,
                  phase: "error",
                  message:
                    error instanceof Error ? error.message : String(error),
                });
              });
            }}
          >
            Export
          </Button>
          <Button
            tone="quiet"
            disabled={!rendering}
            onClick={() => invoke("cancel_render").catch(() => undefined)}
          >
            Cancel
          </Button>
        </div>
        {status?.message ? (
          <p className={styles.meta}>{status.message}</p>
        ) : null}
      </Card>
    </div>
  );
}
