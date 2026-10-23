import { Config } from "@remotion/cli/config";

// JPEG frames keep the render fast; the film is flat colour and hairlines.
Config.setVideoImageFormat("jpeg");
Config.setOverwriteOutput(true);
Config.setConcurrency(4);
