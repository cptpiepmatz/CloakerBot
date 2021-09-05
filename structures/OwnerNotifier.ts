import readDatabase from "../functions/database/readDatabase";
import DatabaseTables from "../enums/DatabaseTables";
import OwnerNotificationsDBData from "../types/database/OwnerNotificationsDBData";
import {ButtonInteraction, Interaction, Message, MessageActionRow, MessageButton, Snowflake} from "discord.js";
import BaseUserDBData from "../types/database/base/BaseUserDBData";

import update200 from "../owner_messages/Update 2.0.0";
import AwaitLock from "await-lock";
import writeDatabase from "../functions/database/writeDatabase";

const ownerMessages = [update200];

export default class OwnerNotifier {

  private ownerMessageStates: Record<Snowflake, {
    id: Snowflake;
    username: string;
    discriminator: string;
    latestMessageIndex: number;
    allowDMs?: boolean;
  }> = {};

  private lock = new AwaitLock();

  private constructor(dbData: OwnerNotificationsDBData[]) {
    for (let date of dbData) {
      this.ownerMessageStates[date.UserID] = {
        id: date.UserID,
        username: date.UserName,
        discriminator: date.UserDiscriminator,
        allowDMs: date.allowDMs,
        latestMessageIndex: date.latestMessageIndex
      };
    }
  }

  static async create(): Promise<OwnerNotifier> {
    const dbData = await readDatabase(DatabaseTables.OWNER_NOTIFICATIONS, "all");
    return new OwnerNotifier(dbData as OwnerNotificationsDBData[]);
  }

  async messageOwner(message: Message): Promise<void> {
    if (!message.guild) return;
    const owner = await message.guild.fetchOwner();
    if (this.ownerMessageStates[owner.id]?.allowDMs ?? true) {
      await this.lock.acquireAsync();
      const currentStates = this.ownerMessageStates[owner.id] ?? {
        id: owner.id
      };
      const latestMessageIndex = currentStates.latestMessageIndex ?? -1;
      if (latestMessageIndex >= ownerMessages.length - 1) return;
      for (
        let i = latestMessageIndex + 1;
        i < ownerMessages.length;
        i++
      ) {
        await owner.send(ownerMessages[i]);
      }
      currentStates.latestMessageIndex = ownerMessages.length - 1;
      currentStates.username = owner.user.username;
      currentStates.discriminator = owner.user.discriminator;
      this.ownerMessageStates[owner.id] = currentStates;

      await this.updateDB(owner.id);
      this.lock.release();
    }
  }

  async unsubscribe(interaction: Interaction) {
    if (!interaction.isButton()) return;
    if (interaction.customId !== "dm-unsub") return;

    await interaction.deferReply();

    const userId = interaction.user.id;
    this.ownerMessageStates[userId] = {
      id: userId,
      username: interaction.user.username,
      discriminator: interaction.user.discriminator,
      latestMessageIndex: this.ownerMessageStates[userId].latestMessageIndex ?? 0,
      allowDMs: false
    }

    await this.updateDB(userId);

    const dmChannel = await interaction.user.createDM();
    const message = await dmChannel.messages.fetch(interaction.message.id)
    message?.edit({components: []});

    await interaction.editReply({
      content: "Unsubscribed successfully"
    });

    await interaction.user.send({
      content: "You're still able to opt in back again.",
      components: [new MessageActionRow().addComponents(new MessageButton({
        customId: "dm-sub",
        label: "Subscribe again",
        style: "PRIMARY"
      }))]
    });
  }

  async subscribe(interaction: Interaction) {
    if (!interaction.isButton()) return;
    if (interaction.customId !== "dm-sub") return;

    await interaction.deferReply();

    const userId = interaction.user.id;
    this.ownerMessageStates[userId] = {
      id: userId,
      username: interaction.user.username,
      discriminator: interaction.user.discriminator,
      latestMessageIndex: this.ownerMessageStates[userId].latestMessageIndex ?? 0,
      allowDMs: true
    }

    await this.updateDB(userId);

    const dmChannel = await interaction.user.createDM();
    const message = await dmChannel.messages.fetch(interaction.message.id)
    message?.edit({components: []});

    await interaction.editReply({
      content: "Subscribed again"
    });

    await interaction.user.send({
      content: "Want to unsubscribe again?",
      components: [new MessageActionRow().addComponents(new MessageButton({
        customId: "dm-unsub",
        label: "Unsubscribe again",
        style: "DANGER"
      }))]
    });
  }

  private async updateDB(ownerId: Snowflake) {
    const collection = dbo.collection(DatabaseTables.OWNER_NOTIFICATIONS);
    const currentState = this.ownerMessageStates[ownerId];
    const insertData: OwnerNotificationsDBData = {
      UserID: ownerId,
      UserName: currentState.username,
      //@ts-ignore I know this doesn't mach exactly here, but it will
      UserDiscriminator: currentState.discriminator,
      latestMessageIndex: currentState.latestMessageIndex,
      allowDMs: currentState.allowDMs
    }
    await collection.updateOne(
      {UserID: ownerId},
      {$set: insertData},
      {upsert: true}
    );
  }
}
