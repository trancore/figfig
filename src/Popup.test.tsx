import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Popup } from "./Popup";
import { emptyPopup } from "./types";

describe("Popup", () => {
  it("shows descriptions and the selection supplied by the shell", () => {
    const candidates = ["checkout", "cherry-pick"].map((label) => ({
      label,
      description: `${label} description`,
      insertText: label,
      kind: "subcommand",
      appendSpace: true,
    }));
    const { rerender } = render(<Popup completion={{ ...emptyPopup, candidates, selected: 0 }} />);
    expect(screen.getByRole("option", { name: /checkout/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByText("checkout description")).toBeVisible();
    expect(screen.getByRole("option", { name: "checkout" })).toHaveAccessibleDescription("checkout description");
    expect(screen.queryByText("cherry-pick description")).not.toBeInTheDocument();
    rerender(<Popup completion={{ ...emptyPopup, candidates, selected: 1 }} />);
    expect(screen.getByRole("option", { name: /cherry-pick/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("option", { name: /checkout/ })).toHaveAttribute("aria-selected", "false");
    expect(screen.getByText("cherry-pick description")).toBeVisible();
    expect(screen.getByRole("option", { name: "cherry-pick" })).toHaveAccessibleDescription("cherry-pick description");
    expect(screen.queryByText("checkout description")).not.toBeInTheDocument();
  });
  it("renders an empty candidate list without a selectable row", () => {
    render(<Popup completion={emptyPopup} />);
    expect(screen.getByRole("listbox")).toBeEmptyDOMElement();
    expect(screen.getByRole("listbox")).not.toHaveAttribute("aria-activedescendant");
  });
  it("shows folder metadata when the selected directory has no description", () => {
    const candidate = { label: "articles/", description: "", insertText: "articles/", kind: "directory", appendSpace: false };
    render(<Popup completion={{ ...emptyPopup, candidates: [candidate] }} />);
    expect(screen.getByRole("option", { name: "articles/" })).toHaveAccessibleDescription("folder");
    expect(screen.getByText("folder")).toBeVisible();
  });
});
