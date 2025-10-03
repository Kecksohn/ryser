import React from "react";

/**
 * Convert ISO country code to flag emoji using Regional Indicator Symbols
 * Supports both modern and historical country codes
 */
export const getCountryFlag = (countryCode) => {
  if (!countryCode || countryCode.length !== 2) {
    return null;
  }

  // Handle special historical cases that don't follow standard conversion
  const historicalFlags = {
    'SU': '🇸🇺', // Soviet Union
    'YU': '🇾🇺', // Yugoslavia
    'CS': '🇨🇸', // Czechoslovakia
    'DD': '🇩🇩', // East Germany
  };

  if (historicalFlags[countryCode.toUpperCase()]) {
    return historicalFlags[countryCode.toUpperCase()];
  }

  // Standard conversion for modern ISO codes
  try {
    const codePoints = countryCode
      .toUpperCase()
      .split('')
      .map(char => 127397 + char.charCodeAt(0));
    return String.fromCodePoint(...codePoints);
  } catch (error) {
    console.warn(`Could not generate flag for country code: ${countryCode}`);
    return null;
  }
};

/**
 * Country name to ISO code mapping for common variations
 * Includes historical country codes and TMDB-specific mappings
 */
const countryNameMapping = {
  // Standard name variations
  'United States': 'US',
  'United Kingdom': 'GB',
  'UK': 'GB',
  'USA': 'US',

  // Historical countries
  'Soviet Union': 'SU',
  'USSR': 'SU',
  'Yugoslavia': 'YU',
  'Czechoslovakia': 'CS',
  'East Germany': 'DD',
  'West Germany': 'DE',
  'German Democratic Republic': 'DD',
  'Federal Republic of Germany': 'DE',

  // TMDB-specific codes that might not match standard ISO
  'XWG': 'DE',  // West Germany historical code
  'XCZ': 'CS',  // Czechoslovakia historical code
  'XYU': 'YU',  // Yugoslavia historical code
  'XSU': 'SU',  // Soviet Union historical code

  // Common country name variations
  'Mexico': 'MX',
  'Germany': 'DE',
  'France': 'FR',
  'Spain': 'ES',
  'Italy': 'IT',
  'Japan': 'JP',
  'China': 'CN',
  'Russia': 'RU',
  'Brazil': 'BR',
  'Canada': 'CA',
  'Australia': 'AU',
  'India': 'IN',
};

/**
 * Convert country name to ISO code if needed
 */
export const normalizeCountryCode = (country) => {
  if (!country) return null;

  // If it's already a 2-letter code, return it
  if (country.length === 2 && /^[A-Z]{2}$/i.test(country)) {
    return country.toUpperCase();
  }

  // If it's a 3-letter historical code, try to map it
  if (country.length === 3 && /^[A-Z]{3}$/i.test(country)) {
    const mapped = countryNameMapping[country.toUpperCase()];
    if (mapped) return mapped;
  }

  // Try to find mapping by name
  const mapped = countryNameMapping[country];
  if (mapped) return mapped;

  // Debug: log unmapped countries
  if (country !== country.toUpperCase() || country.length > 3) {
    console.log(`Unmapped country: "${country}"`);
  }

  return null;
};

/**
 * React component for displaying country flags
 */
export const CountryFlag = ({
  countryCode,
  countryName,
  size = "normal",
  className = "",
  showFallback = true
}) => {
  // Normalize the country code
  const normalizedCode = countryCode ?
    normalizeCountryCode(countryCode) :
    normalizeCountryCode(countryName);

  if (!normalizedCode) {
    return null;
  }

  const flagEmoji = getCountryFlag(normalizedCode);

  if (!flagEmoji) {
    return null;
  }

  const sizeClass = {
    small: 'flag-small',
    normal: 'flag-inline',
    large: 'flag-large'
  }[size] || 'flag-inline';

  return (
    <span
      className={`flag-emoji ${sizeClass} ${className}`}
      title={`Flag of ${countryName || normalizedCode}`}
      role="img"
      aria-label={`Flag of ${countryName || normalizedCode}`}
    >
      {flagEmoji}
    </span>
  );
};

/**
 * Component for displaying multiple country flags inline
 */
export const CountryFlags = ({
  countries,
  size = "normal",
  separator = ", ",
  className = ""
}) => {
  if (!countries || countries.length === 0) {
    return null;
  }

  // Filter out countries that don't render valid flags
  const validFlags = countries
    .map((country, index) => ({
      country,
      index,
      component: (
        <CountryFlag
          key={country + index}
          countryCode={country}
          countryName={country}
          size={size}
        />
      )
    }))
    .filter(item => {
      // Check if the flag component would render something
      const normalizedCode = item.country.length === 2 && /^[A-Z]{2}$/i.test(item.country) ?
        item.country.toUpperCase() :
        countryNameMapping[item.country] || null;
      return normalizedCode && getCountryFlag(normalizedCode);
    });

  if (validFlags.length === 0) {
    return null;
  }

  return (
    <span className={className}>
      {validFlags.map((item, index) => (
        <React.Fragment key={item.country + item.index}>
          {item.component}
          {index < validFlags.length - 1 && (
            <span className="flag-separator">{separator}</span>
          )}
        </React.Fragment>
      ))}
    </span>
  );
};