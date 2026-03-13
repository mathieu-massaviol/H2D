/**
 * Normalizes a HAL identifier from either a plain ID or a full HAL URL.
 * Strips URL prefix and version suffix (v1, v2, etc.).
 *
 * IMPORTANT: This function must stay in sync with the Rust normalize_hal_id
 * function in src-tauri/src/hal_service.rs.
 */
export function normalizeHalId(input: string): string {
  const trimmed = input.trim()

  let idSegment: string
  if (trimmed.startsWith('http')) {
    const parts = trimmed.replace(/\/$/, '').split('/')
    idSegment =
      parts.find(
        (s) => s.startsWith('hal-') || s.startsWith('halshs-') || s.startsWith('hal-shs-'),
      ) ?? trimmed
  } else {
    idSegment = trimmed
  }

  // Strip version suffix (v1, v12, etc.)
  const vIdx = idSegment.lastIndexOf('v')
  if (vIdx !== -1 && /^\d+$/.test(idSegment.slice(vIdx + 1))) {
    return idSegment.slice(0, vIdx)
  }
  return idSegment
}
