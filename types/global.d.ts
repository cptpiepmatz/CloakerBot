import {ApplicationCommandData, Client} from "discord.js";
import {Db} from "mongodb";
import AwaitLock from "await-lock";

declare global {
  namespace globalThis {
    var client: Client;
    var dbo: Db;
    var commands: {
      takedown: ApplicationCommandData,
      random: ApplicationCommandData,
      setRole: ApplicationCommandData,
      ranks: ApplicationCommandData,
      help: ApplicationCommandData
    }
    var takedownLocks: {
      [key: string]: AwaitLock
    }
  }
}

export {}
