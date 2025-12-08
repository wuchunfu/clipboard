import { useI18n } from "vue-i18n";

export function useTimeAgo() {
  const { t } = useI18n();

  function formatTimeAgo(timestamp: string): string {
    if (!timestamp) return "";

    // Handle "YYYY-MM-DD HH:mm:ss" or "YYYY-MM-DD HH:mm:ss.SSS"
    // Replace space with T for better parsing support
    const date = new Date(timestamp.replace(" ", "T"));
    const now = new Date();

    // Check if date is valid
    if (isNaN(date.getTime())) {
      return timestamp;
    }

    const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diffInSeconds < 60) {
      return t("time.justNow");
    }

    const diffInMinutes = Math.floor(diffInSeconds / 60);
    if (diffInMinutes < 60) {
      return t("time.minutesAgo", { n: diffInMinutes });
    }

    const diffInHours = Math.floor(diffInMinutes / 60);
    if (diffInHours < 24) {
      return t("time.hoursAgo", { n: diffInHours });
    }

    const diffInDays = Math.floor(diffInHours / 24);
    if (diffInDays < 30) {
      return t("time.daysAgo", { n: diffInDays });
    }

    const diffInMonths = Math.floor(diffInDays / 30);
    if (diffInMonths < 12) {
      return t("time.monthsAgo", { n: diffInMonths });
    }

    const diffInYears = Math.floor(diffInMonths / 12);
    return t("time.yearsAgo", { n: diffInYears });
  }

  return {
    formatTimeAgo,
  };
}
