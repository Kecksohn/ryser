// Use this script as
// node AddFieldToJson.js "field_name" ((value))
//    ((value)) will be parsed as the correct js type for "string", int, true/false (bool), Date, [array, second].

import { readFileSync, writeFileSync } from "fs";
import fs from "fs";
import path from "path";

/*/ Example usage
const jsonFilePath =
  "C:\\Users\\Prohaska-VCE\\AppData\\Local\\ryser\\data\\dsktp\\library.json";
addFieldToVideos(jsonFilePath, "audio_track_selected", 0);
*/
function addFieldToVideos(jsonFilePath, newField, defaultValue) {
  // Read the JSON file
  const data = JSON.parse(readFileSync(jsonFilePath, "utf8"));

  // Add the new field to each video file object
  data.video_files.forEach((video) => {
    if (!(newField in video)) {
      video[newField] = defaultValue;
    }
  });

  // Write the updated data back to the file
  writeFileSync(jsonFilePath, JSON.stringify(data, null, 2));
}

// Ryser specific logic
const args = process.argv.slice(2);
if (args.length < 2) {
  console.error(
    'Expected at least 2 arguments.\nFirst: "field_name"\nSecond: value',
  );
  process.exit(1); // Exit with non-zero code to indicate failure
}

const field_name = args[0];
const field_value = parseValue(args[1]);

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
        addFieldToVideos(libraryPath, field_name, field_value);
        console.log(
          `Added ${field_name}: ${field_value} (${typeof field_value}) to ${libraryPath}`,
        );
      } else {
        console.warn(`library.json not found in: ${subfolderPath}`);
      }
    }
  }
} else {
  console.error(`Directory does not exist: ${ryserDir}`);
}

function parseValue(value) {
  // TODO: Check '' or ""

  // Check null
  if (value === "null") return null;

  // Check boolean
  if (value.toLowerCase() === "true") return true;
  if (value.toLowerCase() === "false") return false;

  // Check number
  if (!isNaN(value) && value.trim() !== "") {
    // Parse number (int or float)
    return Number(value);
  }

  // Check JSON array/object
  try {
    const parsed = JSON.parse(value);
    if (typeof parsed === "object") {
      return parsed;
    }
  } catch {}

  // Check ISO date string
  const date = Date.parse(value);
  if (!isNaN(date)) {
    return new Date(date).toISOString();
  }

  // fallback: string
  return value;
}
