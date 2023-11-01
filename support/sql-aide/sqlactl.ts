#!/usr/bin/env -S deno run --allow-read --allow-write --allow-env --allow-run --allow-sys

import { cliffy, path, SQLa } from "./deps.ts";
import * as nbooks from "./notebooks.ts";

// deno-lint-ignore no-explicit-any
type Any = any;

async function CLI() {
  const sno = new nbooks.SqlNotebooksOrchestrator(
    new nbooks.SqlNotebookHelpers<SQLa.SqlEmitContext>(),
  );
  const { nbh } = sno;

  const callerName = import.meta.resolve(import.meta.url);
  await new cliffy.Command()
    .name(callerName.slice(callerName.lastIndexOf("/") + 1))
    .version("0.1.0")
    .description("SQL Aide (SQLa) Controller")
    .option(
      "--sql-home <path:string>",
      "Store the generated SQL in the provided directory",
      {
        default: path.relative(
          Deno.cwd(),
          path.fromFileUrl(import.meta.resolve("../../src")),
        ),
      },
    )
    .action(async ({ sqlHome }) => {
      // TODO: figure out how to check whether migrations are done already
      // and pass in the migrations table to the first SQL chains so that
      // they do not perform tasks that have already been performed.

      const sqlPageNB = nbooks.SQLPageNotebook.create(sno.nbh);
      const initSQL = nbh.SQL`
        ${sno.constructionNB.bootstrapDDL()}

        -- store all SQL that is potentially reusable in the database
        ${await sno.storeNotebookCellsDML()}

        -- insert SQLPage content for diagnostics and web server
        ${await sqlPageNB.SQL()}
        `;

      await Deno.writeTextFile(
        path.join(sqlHome, "bootstrap.sql"),
        initSQL.SQL(sno.nbh.emitCtx),
      );
    })
    .command("help", new cliffy.HelpCommand().global())
    .command("completions", new cliffy.CompletionsCommand())
    .command(
      "diagram",
      new cliffy.Command()
        .description("Emit Diagram")
        .option(
          "-d, --dest <file:string>",
          "Output destination, STDOUT if not supplied",
        )
        .action((options) => {
          const diagram = sno.infoSchemaDiagram();
          if (options.dest) {
            Deno.writeTextFileSync(options.dest, diagram);
          } else {
            console.log(diagram);
          }
        }),
    )
    .parse();
}

if (import.meta.main) {
  await CLI();
}
