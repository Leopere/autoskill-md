export function checkReadability(markdown, maxGrade = 7) {
  const grade = fleschKincaidGrade(markdown);
  return {
    grade,
    maxGrade,
    ok: grade <= maxGrade
  };
}

export function fleschKincaidGrade(markdown) {
  const text = normalize(markdown);
  const sentences = Math.max(1, (text.match(/[.!?]+(?:\s|$)/g) ?? []).length);
  const words = text.match(/[A-Za-z]+(?:'[A-Za-z]+)?/g) ?? [];
  if (words.length === 0) return 0;

  const syllables = words.reduce((count, word) => count + countSyllables(word), 0);
  const grade = 0.39 * (words.length / sentences) + 11.8 * (syllables / words.length) - 15.59;
  return Math.max(0, Number(grade.toFixed(2)));
}

function normalize(markdown) {
  return markdown
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`[^`]*`/g, " ")
    .replace(/https?:\/\/\S+/g, " ")
    .replace(/[#>*_[\]()-]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function countSyllables(word) {
  const clean = word.toLowerCase().replace(/[^a-z]/g, "");
  if (!clean) return 0;
  if (clean.length <= 3) return 1;

  const withoutSilentE = clean.replace(/e\b/, "");
  const groups = withoutSilentE.match(/[aeiouy]+/g);
  return Math.max(1, groups ? groups.length : 1);
}
