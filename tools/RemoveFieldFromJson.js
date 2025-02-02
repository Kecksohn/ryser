import { readFileSync, writeFileSync } from 'fs';

function removeFieldFromVideos(jsonFilePath, fieldToRemove) {
	// Read the JSON file
	const data = JSON.parse(readFileSync(jsonFilePath, 'utf8'));

	// Remove the field from each video file object
	data.video_files.forEach(video => {
		if (fieldToRemove in video) {
			delete video[fieldToRemove];
		}
	});

	// Write the updated data back to the file
	writeFileSync(jsonFilePath, JSON.stringify(data, null, 2));
}

// Example usage
const jsonFilePath = 'C:\\Users\\kecks\\AppData\\Local\\ryser\\data\\movies\\library.json';
removeFieldFromVideos(jsonFilePath, 'date_added');