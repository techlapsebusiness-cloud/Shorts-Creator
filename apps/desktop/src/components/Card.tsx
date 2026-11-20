import type { ReactNode } from "react";
import styles from "./Card.module.css";

type CardProps = {
  title: string;
  children: ReactNode;
  className?: string;
};

export function Card({ title, children, className }: CardProps) {
  return (
    <section className={[styles.card, className].filter(Boolean).join(" ")}>
      <h2 className={styles.title}>{title}</h2>
      {children}
    </section>
  );
}
