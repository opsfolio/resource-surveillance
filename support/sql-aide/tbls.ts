import * as yaml from "https://deno.land/std@0.206.0/yaml/stringify.ts";

export interface TblsConfig extends Record<string, unknown> {
  name?: string;
  format?: Format;
  er?: EntityRelDiagram;
  desc?: string;
  labels?: string[];
  comments?: TblsCommentConfig[];
  include?: string[];
  exclude?: string[];
}

export interface EntityRelDiagram {
  comment?: boolean;
  hideDef?: boolean;
  distance?: number;
}

export interface Format {
  adjust: boolean;
  hideColumnsWithoutValues: string[];
}

export interface TblsCommentConfig {
  table: string;
  tableComment?: string;
  columnComments?: Record<string, string>;
  labels?: string[];
  columnLabels?: Record<string, string>;
}

export function tblsConfig(tc: TblsConfig) {
  return yaml.stringify(tc);
}
