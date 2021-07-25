import {Client, Intents} from "discord.js";

globalThis.client = new Client({
  messageCacheLifetime: 3 * 60,
  messageSweepInterval: 30,
  intents: [
    Intents.FLAGS.GUILDS,
    Intents.FLAGS.GUILD_VOICE_STATES,
    Intents.FLAGS.GUILD_MESSAGES,
    Intents.FLAGS.GUILD_INTEGRATIONS
  ]
});
