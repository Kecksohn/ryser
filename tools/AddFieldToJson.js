import { readFileSync, writeFileSync } from 'fs';

function addFieldToVideos(jsonFilePath, newField, defaultValue) {
	// Read the JSON file
	const data = JSON.parse(readFileSync(jsonFilePath, 'utf8'));

	// Add the new field to each video file object
	data.video_files.forEach(video => {
		if (!(newField in video)) {
			video[newField] = defaultValue;
		}
	});

	// Write the updated data back to the file
	writeFileSync(jsonFilePath, JSON.stringify(data, null, 2));
}

// Example usage
const jsonFilePath = 'C:\\Users\\kecks\\AppData\\Local\\ryser\\data\\movies\\library.json';
addFieldToVideos(jsonFilePath, 'timestamp_modified', Date.now());