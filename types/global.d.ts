import {Client} from "discord.js";
import {Db} from "mongodb";

declare global {
  namespace globalThis {
    var client: Client;
    var dbo: Db;
  }
}

export {}
