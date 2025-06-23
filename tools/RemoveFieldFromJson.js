// Use this script as
// node RemoveFieldFromJson.js "field_name"

import { readFileSync, writeFileSync } from "fs";
import fs from "fs";
import path from "path";

/*/ Example usage
const jsonFilePath =
  "C:\\Users\\Prohaska-VCE\\AppData\\Local\\ryser\\data\\dsaas2\\library.json";
removeFieldFromVideos(jsonFilePath, "audio_track_selected"); */
function removeFieldFromVideos(jsonFilePath, fieldToRemove) {
  // Read the JSON file
  const data = JSON.parse(readFileSync(jsonFilePath, "utf8"));

  // Remove the field from each video file object
  data.video_files.forEach((video) => {
    if (fieldToRemove in video) {
      delete video[fieldToRemove];
    }
  });

  // Write the updated data back to the file
  writeFileSync(jsonFilePath, JSON.stringify(data, null, 2));
}

// Ryser specific logic
const args = process.argv.slice(2);
if (args.length < 1) {
  console.error('Expected at least 1 argument: "field_name"');
  process.exit(1); // Exit with non-zero code to indicate failure
}
const field_name = args[0];

const localAppData = process.env.LOCALAPPDATA;
const ryserDir = path.join(localAppData, "ryser", "data");

// Function that processes the parsed JSON
function handleLibraryData(data, folderName) {
  console.log(`Processing ${folderName}:`, data);
  // Your custom logic here
}

// Check if ryser directory exists
if (fs.existsSync(ryserDir)) {
  // Read all subfolders
  const entries = fs.readdirSync(ryserDir, { withFileTypes: true });

  for (const entry of entries) {
    if (entry.isDirectory()) {
      const subfolderPath = path.join(ryserDir, entry.name);
      const libraryPath = path.join(subfolderPath, "library.json");
      if (fs.existsSync(libraryPath)) {
        removeFieldFromVideos(libraryPath, field_name);
        console.log(`Removed ${field_name}: from ${libraryPath}`);
      } else {
        console.warn(`library.json not found in: ${subfolderPath}`);
      }
    }
  }
} else {
  console.error(`Directory does not exist: ${ryserDir}`);
}
