export interface Candidate {
  label: string;
  description: string;
  insertText: string;
  kind: string;
  appendSpace: boolean;
}
export interface CompletionResponse {
  sessionId: string;
  revision: number;
  replaceStart: number;
  replaceEnd: number;
  candidates: Candidate[];
  selected: number;
  visible: boolean;
}
export const emptyPopup: CompletionResponse = {
  sessionId: "",
  revision: 0,
  replaceStart: 0,
  replaceEnd: 0,
  candidates: [],
  selected: 0,
  visible: false,
};
