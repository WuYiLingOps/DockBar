import { UIColors } from "../../system_state/ui_colors.ts";
import { RuntimeStyleSheet } from "../../utils/DOM.ts";

/**
 * Applies the system UI colors (accent palette) as CSS variables and keeps
 * date-related CSS variables up to date. This replaces the old theming tool:
 * widget styling itself ships as static CSS bundled with each widget.
 */
export async function startAppearanceTool(): Promise<void> {
  (await UIColors.getAsync()).setAsCssVariables();
  await UIColors.onChange((colors) => colors.setAsCssVariables());

  startDateCssVariables();
}

function startDateCssVariables(): void {
  // Set initial values immediately
  updateDateCssVariables();
  // Update every minute (60000ms) to avoid overhead from seconds
  setInterval(updateDateCssVariables, 60000);
}

function updateDateCssVariables(): void {
  const now = new Date();
  const locale = navigator.language;

  // Time values
  const hour = now.getHours();
  const minute = now.getMinutes();

  // Date name values using Intl API
  const dayName = new Intl.DateTimeFormat(locale, { weekday: "long" }).format(now);
  const monthName = new Intl.DateTimeFormat(locale, { month: "long" }).format(now);

  // Date numeric values
  const dayOfMonth = now.getDate();
  const monthNumber = now.getMonth() + 1;
  const year = now.getFullYear();

  const styleSheet = new RuntimeStyleSheet("@runtime/date-variables");

  // Time variables
  styleSheet.addVariable("--date-hour", String(hour));
  styleSheet.addVariable("--date-minute", String(minute));

  // Date name variables (localized)
  styleSheet.addVariable("--date-day-name", dayName);
  styleSheet.addVariable("--date-month-name", monthName);

  // Date numeric values
  styleSheet.addVariable("--date-day", String(dayOfMonth));
  styleSheet.addVariable("--date-month", String(monthNumber));
  styleSheet.addVariable("--date-year", String(year));

  styleSheet.applyToDocument();
}
