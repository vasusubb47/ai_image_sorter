import express, { Express, Request, Response } from "express";
import fsPros from "fs/promises";
import fs from "fs";
import z from "zod";

const app: Express = express();

const port = process.env.PORT || 8888;

app.set("view engine", "ejs");

app.get("/data", async (req: Request, res: Response) => {
  const data = JSON.parse(
    (await fsPros.readFile("./proj.env.json")).toString()
  );
  console.log(data);

  const projs = fs
    .readdirSync(data.projectDataLocation, {
      withFileTypes: true,
    })
    .filter((item) => item.isDirectory());

  res.json({ data, projs });
});

app.get("/", async (req: Request, res: Response) => {
  res.render("index");
});

app.get("/new", async (req: Request, res: Response) => {
  res.render("newProject");
});

app.listen(port, () => {
  console.log(`⚡️[server]: Server is running at http://localhost:${port}`);
});
