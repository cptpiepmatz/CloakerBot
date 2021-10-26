import {ApplicationCommandData, ChatInputApplicationCommandData, ContextMenuInteraction} from "discord.js";
import {ApplicationCommandTypes} from "discord.js/typings/enums";
import getRandom from "../../functions/util/getRandom";
import replies from "../../provider/replies";
import {commandTakedownPermCheck, tryTakedownMember} from "../takedown";

async function takedownContextCommand(interaction: ContextMenuInteraction) {
  if (!await commandTakedownPermCheck(interaction)) return;
  let member = await interaction.guild?.members.fetch(interaction.targetId);
  if (!member) {
    await interaction.reply({
      content: getRandom(replies.noMember),
      ephemeral: true
    });
    return;
  }
  await tryTakedownMember(member, interaction);
}

const commandData: Exclude<
  ApplicationCommandData, ChatInputApplicationCommandData
> = {
  name: "takedown",
  type: ApplicationCommandTypes.USER
}

takedownContextCommand.commandData = commandData;

export default takedownContextCommand;

