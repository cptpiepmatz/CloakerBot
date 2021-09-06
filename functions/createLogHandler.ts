import Logger, {ILogHandler, ILogger} from "js-logger";
import {blue, bold, cyan, gray, green, greenBright, magenta, red, white, yellow} from "chalk";
import callsites from "callsites";
import {inspect} from "util";
import {extname, basename, relative, join} from "path";

const createLogHandler: ILogHandler = function(messages, context) {
  let callPath = "";
  for (let callsite of callsites()) {
    let fileName = callsite.getFileName();
    if (fileName && !fileName.includes("node_modules") && !fileName.includes("node:")) {
      callPath = relative(join(__dirname, "../"), fileName);
    }
  }

  let base = callPath.substring(0, callPath.length - 3);

  let date = new Date();
  function twoDigits(number: Number) {return `0${number}`.substr(-2)}
  let hours = twoDigits(date.getHours());
  let minutes = twoDigits(date.getMinutes());
  let seconds = twoDigits(date.getSeconds());
  let timestamp =
    white("[") +
    cyan(`${hours}:${minutes}:${seconds}`) +
    white("]");

  let name = context.name ?? base;

  let message = `${timestamp} <${blue(name)}> ${gray(context.level.name)} `;

  let content = messages[0];
  if (typeof content !== "string") content = "\n" + white(inspect(content, {
    colors: true,
    depth: 3
  }));

  switch (context.level.name) {
    case "TRACE": return console.log(message + gray(content));
    case "DEBUG": return console.debug(message + white(content));
    case "INFO": return console.info(message + greenBright(content));
    case "WARN": return console.warn(message + yellow(content));
    case "ERROR": return console.error(message + red(content));
  }

  Logger.createDefaultHandler()(messages, context);
}

export default createLogHandler;
