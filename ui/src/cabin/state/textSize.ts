/**
 * The text sizes the player chooses from (Settings › Appearance › Text Size), nine steps, each a scale of the apps'
 * text (their `font-size`s times `--text-scale`). Only the text: icons, artwork, keys and spacing stay as they are
 * (owner 2026-10-22). Larger than iOS's own (its 0.82 to 1.35 about the default): a screen in a cab, seen through the
 * game, needs larger text, and that range was too thin to tell (owner 2026-10-24: "the texts are too small. Move the
 * text size range to larger side overall"): up to twice the apps' text as designed, as iOS's larger accessibility
 * sizes go. And down to 0.8 of it, iOS's own small end, for a screen seen larger or closer than the DAF's (owner
 * 2026-10-24: "Extend the range of text size").
 */
export const TEXT_SIZES = [0.8, 0.9, 1, 1.12, 1.24, 1.35, 1.5, 1.7, 2] as const

/** The step TruckersTV starts on: a quarter larger than the apps' text as designed. */
export const DEFAULT_TEXT_SIZE = 4

/** A step kept or sent, as one there is (anything else: the default). */
export function toTextSize(value: unknown): number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 0 && value < TEXT_SIZES.length
    ? value
    : DEFAULT_TEXT_SIZE
}

/** How much a step scales the apps' text. */
export function textScale(step: number): number {
  return TEXT_SIZES[toTextSize(step)]
}
