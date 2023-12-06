import { chainNB, safety, SQLa } from "./deps.ts";

// deno-lint-ignore no-explicit-any
type Any = any;

export interface PolygenSrcCodeEmitOptions {
  readonly tableStructName: (tableName: string) => string;
  readonly tableStructFieldName: (
    tc: { tableName: string; columnName: string },
  ) => string;
}

export interface PolygenEmitContext {
  readonly pscEmitOptions: PolygenSrcCodeEmitOptions;
}

export type PolygenSrcCodeText =
  | string
  | string[];

export type PolygenSrcCode<Context extends PolygenEmitContext> =
  | PolygenSrcCodeText
  | ((ctx: Context) => PolygenSrcCodeText | Promise<PolygenSrcCodeText>);

export async function sourceCodeText<Context extends PolygenEmitContext>(
  ctx: Context,
  psc: PolygenSrcCodeSupplier<Context> | PolygenSrcCode<Context>,
): Promise<string> {
  if (isPolygenSrcCodeSupplier<Context>(psc)) {
    return sourceCodeText(ctx, psc.sourceCode);
  }

  if (typeof psc === "string") {
    return psc;
  } else if (typeof psc === "function") {
    return await sourceCodeText(ctx, await psc(ctx));
  } else {
    if (psc.length == 0) return "";
    return psc.join("\n");
  }
}

export interface PolygenSrcCodeSupplier<Context extends PolygenEmitContext> {
  readonly sourceCode: PolygenSrcCode<Context>;
}

export interface PolygenEngine<
  PolygenContext extends PolygenEmitContext,
  SqlContext extends SQLa.SqlEmitContext,
  DomainQS extends SQLa.SqlDomainQS,
  DomainsQS extends SQLa.SqlDomainsQS<DomainQS>,
> {
  readonly polygenEmitCtx: () => PolygenContext;
  readonly entityAttrSrcCode: (
    ea: SQLa.GraphEntityAttrReference<
      Any,
      Any,
      SqlContext,
      DomainQS,
      DomainsQS
    >,
  ) => PolygenSrcCode<PolygenContext>;
  readonly entitySrcCode: (
    e: SQLa.GraphEntityDefinition<Any, SqlContext, Any, DomainQS, DomainsQS>,
  ) => PolygenSrcCode<PolygenContext>;
}

export function isPolygenSrcCodeSupplier<Context extends PolygenEmitContext>(
  o: unknown,
): o is PolygenSrcCodeSupplier<Context> {
  const isPSCS = safety.typeGuard<PolygenSrcCodeSupplier<Context>>(
    "sourceCode",
  );
  return isPSCS(o);
}

export interface PolygenSrcCodeBehaviorEmitTransformer {
  before: (interpolationSoFar: string, exprIdx: number) => string;
  after: (nextLiteral: string, exprIdx: number) => string;
}

export const removeLineFromEmitStream: PolygenSrcCodeBehaviorEmitTransformer = {
  before: (isf) => {
    // remove the last line in the interpolation stream
    return isf.replace(/\n.*?$/, "");
  },
  after: (literal) => {
    // remove everything up to and including the line break
    return literal.replace(/.*?\n/, "\n");
  },
};

export interface PolygenSrcCodeBehaviorSupplier<
  Context extends PolygenEmitContext,
> {
  readonly executePolygenSrcCodeBehavior: (
    context: Context,
  ) =>
    | PolygenSrcCodeBehaviorEmitTransformer
    | PolygenSrcCodeSupplier<Context>
    | PolygenSrcCodeSupplier<Context>[];
}

export function isPolygenSrcCodeBehaviorSupplier<
  Context extends PolygenEmitContext,
>(
  o: unknown,
): o is PolygenSrcCodeBehaviorSupplier<Context> {
  const isPSCBS = safety.typeGuard<
    PolygenSrcCodeBehaviorSupplier<Context>
  >("executePolygenSrcCodeBehavior");
  return isPSCBS(o);
}

/**
 * Chain-of-Responsiblity style notebook base class
 */
export abstract class PolygenNotebook<PolygenEmitContext> {
}

export function polygenlNotebookAnnotations<
  Notebook extends PolygenNotebook<Context>,
  Context extends PolygenEmitContext,
>() {
  return new chainNB.NotebookDescriptor<
    Notebook,
    chainNB.NotebookCell<Notebook, chainNB.NotebookCellID<Notebook>>
  >();
}

export function polygenNotebookFactory<
  Notebook extends PolygenNotebook<Context>,
  Context extends PolygenEmitContext,
>(
  prototype: Notebook,
  instance: () => Notebook,
  nbd = polygenlNotebookAnnotations<Notebook, Context>(),
) {
  type CellID = chainNB.NotebookCellID<Notebook>;
  const kernel = chainNB.ObservableKernel.create(prototype, nbd);

  type EventEmitter = Awaited<
    ReturnType<typeof kernel.initRunState>
  >["runState"]["eventEmitter"];
  return {
    nbd,
    kernel,
    instance,
    sourceCode: async (
      options: {
        separator?: (
          cell: Parameters<EventEmitter["afterCell"]>[0],
          state: Parameters<EventEmitter["afterCell"]>[1],
        ) => PolygenSrcCodeBehaviorSupplier<Context>;
        onNotSrcCodeSupplier?: (
          cell: Parameters<EventEmitter["afterCell"]>[0],
          state: Parameters<EventEmitter["afterCell"]>[1],
        ) => PolygenSrcCodeBehaviorSupplier<Context>;
      },
      ...srcCodeIdentities: CellID[]
    ) => {
      // prepare the run state with either a list of sql identities if passed
      // or all cells if no specific cells requested
      const initRunState = await kernel.initRunState({
        executeCells: (inb) => {
          if (srcCodeIdentities.length == 0) return inb.cells;
          const specific = srcCodeIdentities.map((si) =>
            inb.cells.find((c) => c.nbCellID == si)
          ).filter((c) => c != undefined) as typeof inb.cells;
          if (specific.length > 0) return specific;
          return inb.cells;
        },
      });

      const sourceCode: (
        | PolygenSrcCodeSupplier<Context>
        | PolygenSrcCodeBehaviorSupplier<Context>
      )[] = [];
      initRunState.runState.eventEmitter.afterCell = (cell, state) => {
        if (state.status == "successful") {
          if (
            isPolygenSrcCodeSupplier<Context>(state.execResult) ||
            isPolygenSrcCodeBehaviorSupplier<Context>(state.execResult)
          ) {
            if (options.separator) {
              sourceCode.push(options.separator(cell, state));
            }
            const sts = state.execResult as PolygenSrcCodeSupplier<Context>;
            sourceCode.push(sts);
          } else {
            const notSTS = options.onNotSrcCodeSupplier?.(cell, state);
            if (notSTS) sourceCode.push(notSTS);
          }
        }
      };
      await kernel.run(instance(), initRunState);
      return sourceCode;
    },
  };
}

export interface PolygenInfoSchemaOptions<
  PolygenContext extends PolygenEmitContext,
  SqlContext extends SQLa.SqlEmitContext,
  DomainQS extends SQLa.SqlDomainQS,
  DomainsQS extends SQLa.SqlDomainsQS<DomainQS>,
> {
  readonly includeEntityAttr: (
    ea: SQLa.GraphEntityAttrReference<
      Any,
      Any,
      SqlContext,
      DomainQS,
      DomainsQS
    >,
  ) => boolean;
  readonly includeEntity: (
    e: SQLa.GraphEntityDefinition<Any, SqlContext, Any, DomainQS, DomainsQS>,
  ) => boolean;
  readonly includeRelationship: (
    edge: SQLa.GraphEdge<SqlContext, Any, Any>,
  ) => boolean;
  readonly includeChildren: (
    ir: SQLa.EntityGraphInboundRelationship<
      Any,
      Any,
      SqlContext,
      DomainQS,
      DomainsQS
    >,
  ) => boolean;
}

export function typicalPolygenInfoSchemaOptions<
  PolygenContext extends PolygenEmitContext,
  SqlContext extends SQLa.SqlEmitContext,
  DomainQS extends SQLa.SqlDomainQS,
  DomainsQS extends SQLa.SqlDomainsQS<DomainQS>,
>(
  inherit?: Partial<
    PolygenInfoSchemaOptions<PolygenContext, SqlContext, DomainQS, DomainsQS>
  >,
): PolygenInfoSchemaOptions<PolygenContext, SqlContext, DomainQS, DomainsQS> {
  // we let type inference occur so generics can follow through
  return {
    includeEntity: () => true,
    includeEntityAttr: () => true,
    includeRelationship: () => true,
    includeChildren: () => true,
    ...inherit,
  };
}

export class RustPolygenEngine<
  PolygenContext extends PolygenEmitContext,
  SqlContext extends SQLa.SqlEmitContext,
  DomainQS extends SQLa.SqlDomainQS,
  DomainsQS extends SQLa.SqlDomainsQS<DomainQS>,
> implements PolygenEngine<PolygenContext, SqlContext, DomainQS, DomainsQS> {
  readonly sqlNames: SQLa.SqlObjectNames;
  #emitCtx = {
    pscEmitOptions: {
      tableStructFieldName: (table) => table.columnName,
      tableStructName: (table) => table,
    },
  } as PolygenContext;

  constructor(
    readonly sqlCtx: SqlContext,
    readonly polygenSchemaOptions: PolygenInfoSchemaOptions<
      PolygenContext,
      SqlContext,
      DomainQS,
      DomainsQS
    >,
  ) {
    this.sqlNames = sqlCtx.sqlNamingStrategy(sqlCtx);
  }

  polygenEmitCtx() {
    return this.#emitCtx;
  }

  sqliteTypeToRustType(sqliteType: string): string {
    switch (sqliteType.toLowerCase()) {
      case "integer":
        return "i64"; // SQLite's INTEGER is a 64-bit signed integer
      case "real":
        return "f64"; // REAL in SQLite is a floating point value
      case "text":
        return "String"; // TEXT maps to Rust's String type
      case "blob":
        return "Vec<u8>"; // BLOB is best represented as a byte array
      case "boolean":
        return "bool"; // Boolean type (commonly stored as INTEGER in SQLite)
      case "date":
        return "chrono::NaiveDate"; // Using chrono crate for date
      case "datetime":
        return "chrono::NaiveDateTime"; // Using chrono crate for datetime
      default:
        return "String"; // Default or unknown types can be mapped to String
    }
  }

  entityAttrSrcCode(
    ea: SQLa.GraphEntityAttrReference<
      Any,
      Any,
      SqlContext,
      DomainQS,
      DomainsQS
    >,
  ) {
    const name = this.sqlNames.tableColumnName({
      tableName: ea.entity.identity("presentation"),
      columnName: ea.attr.identity,
    });
    const descr = SQLa.isTablePrimaryKeyColumnDefn(ea.attr)
      ? `PRIMARY KEY`
      : "";
    const sqlType = this.sqliteTypeToRustType(
      ea.attr.sqlDataType("create table column").SQL(
        this.sqlCtx,
      ),
    );
    // deno-fmt-ignore
    return `    ${name}: ${ea.attr.isNullable() ? `Some(${sqlType})` : sqlType},${descr ? ` // ${descr}` : ''}`;
  }

  entitySrcCode(
    e: SQLa.GraphEntityDefinition<Any, SqlContext, Any, DomainQS, DomainsQS>,
  ) {
    const columns: string[] = [];
    // we want to put all the primary keys at the top of the entity
    for (const column of e.attributes) {
      const ea = { entity: e, attr: column };
      if (!this.polygenSchemaOptions.includeEntityAttr(ea)) continue;
      if (SQLa.isTablePrimaryKeyColumnDefn(column)) {
        columns.push(this.entityAttrSrcCode(ea));
      }
    }

    for (const column of e.attributes) {
      if (!SQLa.isTablePrimaryKeyColumnDefn(column)) {
        const ea = { entity: e, attr: column };
        if (!this.polygenSchemaOptions.includeEntityAttr(ea)) continue;
        columns.push(this.entityAttrSrcCode(ea));
      }
    }

    const structName = this.sqlNames.tableName(e.identity("presentation"));
    return [
      `#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]`,
      `pub struct ${structName} {`,
      ...columns,
      `}`,
      "",
    ];
  }
}

/**
 * Encapsulates polyglot source code generation code.
 */
export class PolygenInfoSchemaNotebook<
  PolygenContext extends PolygenEmitContext,
  Entity extends SQLa.GraphEntityDefinition<
    Any,
    SqlContext,
    Any,
    DomainQS,
    DomainsQS
  >,
  SqlContext extends SQLa.SqlEmitContext,
  DomainQS extends SQLa.SqlDomainQS,
  DomainsQS extends SQLa.SqlDomainsQS<DomainQS>,
> extends PolygenNotebook<PolygenContext> {
  constructor(
    readonly engine: PolygenEngine<
      PolygenContext,
      SqlContext,
      DomainQS,
      DomainsQS
    >,
    readonly sqlCtx: SqlContext,
    readonly entityDefns: (ctx: SqlContext) => Generator<Entity>,
    readonly polygenSchemaOptions: PolygenInfoSchemaOptions<
      PolygenContext,
      SqlContext,
      DomainQS,
      DomainsQS
    >,
  ) {
    super();
  }

  async entitiesSrcCode() {
    const peCtx = this.engine.polygenEmitCtx();
    const graph = SQLa.entitiesGraph(this.sqlCtx, this.entityDefns);

    const entitiesSrcCode: string[] = [];
    for (const entity of graph.entities) {
      if (!this.polygenSchemaOptions.includeEntity(entity)) {
        continue;
      }

      const sc = this.engine.entitySrcCode(entity);
      entitiesSrcCode.push(await sourceCodeText(peCtx, sc));
    }

    const pscSupplier: PolygenSrcCodeSupplier<PolygenContext> = {
      sourceCode: () => {
        return entitiesSrcCode.join("\n");
      },
    };
    return pscSupplier;
  }
}
