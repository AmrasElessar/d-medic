/** Backend `verification::schema` ile birebir uyumlu tipler. */

export type SourceType =
  | 'microsoft'
  | 'cis'
  | 'nist'
  | 'nsa'
  | 'disa'
  | 'github'
  | 'forum'
  | 'mvp'
  | 'other';

export type SourceStance =
  | 'recommends'
  | 'documents'
  | 'supports'
  | 'silent'
  | 'discourages'
  | 'blocks';

export type VerificationLevel =
  | 'safe'
  | 'documented_alternative'
  | 'tried_not_official'
  | 'not_recommended';

export interface VerificationSource {
  type: SourceType;
  url?: string | null;
  reference?: string | null;
  stars?: number | null;
  stance: SourceStance;
  note?: string | null;
  last_verified?: string | null;
}

export interface VerificationRecord {
  verification_level: VerificationLevel;
  harm_record?: string | null;
  sources: VerificationSource[];
  last_audit_date?: string | null;
}
