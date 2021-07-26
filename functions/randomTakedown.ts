import queue from "queue";

import DatabaseTables from "../enums/DatabaseTables";
import RandomTakedownsDBData from "../types/database/RandomTakedownsDBData";
import Stats from "../enums/Stats";
import incStat from "./incStat";
import readDatabase from "./database/readDatabase";
import updateTakedownDate from "./updateTakedownDate";
import takedown from "./takedown";

const collectionName = DatabaseTables.RANDOM_TAKEDOWNS;

function randomTakedown(): void {
  console.log("Initialize random takedowns");

  let kickQueue = queue({autostart: true});

  setInterval(async () =>  {
    let guildData = await getGuildsToTakedown();
    for (let guildDate of guildData) {
      kickQueue.push(async () => {
        let guild = await client.guilds.resolve(guildDate.GuildID);
        if (!guild) return;
        console.log("Random Takedown: " + guild.name);
        let voices = guild.voiceStates;
        if (voices.cache.size >= 3) {
          try {
            let member;
            do {
              member = voices.cache.random().member
            }
            while (member?.id === client?.user?.id);
            if (member && await takedown(member)) {
              incStat(Stats.RANDOM_TAKEDOWNS, guild)
                .catch(console.error);
            }
          }
          catch (e) {
            console.error(e);
          }
        }

        await updateTakedownDate(guild, guildDate);
      });
    }
  }, 120000);

  async function getGuildsToTakedown(): Promise<RandomTakedownsDBData[]> {
    return await readDatabase(
      collectionName,
      "all",
      {
        doRandomTakedowns: true,
        takedownDate: {
          $lte: new Date()
        }
      }
    ) as RandomTakedownsDBData[];
  }
}
