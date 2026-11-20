import type { ButtonHTMLAttributes } from "react";
import styles from "./Button.module.css";

type Tone = "primary" | "quiet" | "danger";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  tone?: Tone;
};

export function Button({
  tone = "quiet",
  className,
  type = "button",
  ...props
}: ButtonProps) {
  const toneClass = tone === "quiet" ? "" : styles[tone];
  return (
    <button
      type={type}
      className={[styles.button, toneClass, className]
        .filter(Boolean)
        .join(" ")}
      {...props}
    />
  );
}
