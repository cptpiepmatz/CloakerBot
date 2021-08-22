/// <reference types="../types/global" />
import {Client, Intents} from "discord.js";
import token from "../provider/token";
import takedownCommand from "../commands/takedown";
import randomCommand from "../commands/random";
import setRoleCommand from "../commands/takedownRole";
import ranksCommand from "../commands/ranks";
import helpCommand from "../commands/help";
import ora from "ora";

const spinner = ora();

(async function() {
  spinner.start("Creating client...");
  const client = new Client({intents: [Intents.FLAGS.GUILD_INTEGRATIONS]});
  spinner.succeed("Created client");

  spinner.start("Logging in...");
  await client.login(token);
  spinner.succeed("Logged in");

  spinner.start("Registering commands...");
  await client.application?.commands.set([
    takedownCommand.commandData,
    randomCommand.commandData,
    setRoleCommand.commandData,
    ranksCommand.commandData,
    helpCommand.commandData
  ]);
  spinner.succeed("Registered commands");

  client.destroy();
})();
