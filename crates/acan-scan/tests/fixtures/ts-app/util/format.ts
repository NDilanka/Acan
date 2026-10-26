export function format(title: string): string {
  return pad(title.trim());
}

function pad(text: string): string {
  return `- ${text}`;
}
