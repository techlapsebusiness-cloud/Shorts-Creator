import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it } from "vitest";
import { Field, SelectField } from "./Field";

function TextHarness() {
  const [value, setValue] = useState("crash-safe saves");
  return <Field label="Topic" value={value} onChange={setValue} />;
}

describe("Field", () => {
  it("associates the label and accepts typing", () => {
    render(<TextHarness />);
    const input = screen.getByRole("textbox", { name: "Topic" });
    expect(input).toHaveValue("crash-safe saves");
    fireEvent.change(input, { target: { value: "ffmpeg filters" } });
    expect(input).toHaveValue("ffmpeg filters");
  });

  it("changes a select", () => {
    function Harness() {
      const [value, setValue] = useState("Hinglish");
      return (
        <SelectField
          label="Language"
          value={value}
          options={["Hinglish", "English"]}
          onChange={setValue}
        />
      );
    }
    render(<Harness />);
    fireEvent.change(screen.getByRole("combobox", { name: "Language" }), {
      target: { value: "English" },
    });
    expect(screen.getByRole("combobox", { name: "Language" })).toHaveValue(
      "English",
    );
  });
});
