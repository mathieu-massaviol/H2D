import type { HalMetadata } from '../components/doi/steps/SelectDocumentStep.vue'

export type BatchDocumentStatus =
  | 'pending'
  | 'loading'
  | 'ready'
  | 'has_doi'
  | 'not_found'
  | 'fetch_error'
  | 'processing'
  | 'success'
  | 'error'

export interface BatchDocument {
  halId: string
  status: BatchDocumentStatus
  selected: boolean
  halMetadata: HalMetadata | null
  overviewError: string | null
  doi: string | null
  halSynced: boolean
  processError: string | null
}

/** Maps a post-processing BatchDocumentStatus to a PrimeVue Tag severity. */
export function batchStatusSeverity(status: BatchDocumentStatus) {
  switch (status) {
    case 'success':
      return 'success'
    case 'error':
      return 'danger'
    case 'has_doi':
    case 'fetch_error':
      return 'warn'
    default:
      return 'secondary'
  }
}
