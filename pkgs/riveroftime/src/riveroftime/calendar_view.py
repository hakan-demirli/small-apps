import calendar
import datetime

RESET = "\033[0m"
BOLD = "\033[1m"

C_PAST = "\033[90m"
C_WEEKEND = "\033[38;5;246m"
C_TODAY_BG = "\033[7m"
C_HEADER_CUR = "\033[1;32m"
C_HEADER_OTH = "\033[1;90m"
C_DAYS_LABEL = "\033[34m"
C_WE_LABEL = "\033[37m"


def run():
    now = datetime.date.today()
    cal = calendar.Calendar(firstweekday=0)

    months = []
    y, m = now.year, now.month
    for _ in range(3):
        months.append((y, m))
        m += 1
        if m > 12:
            m = 1
            y += 1

    grid_headers = []
    grid_days = []
    grid_weeks = []

    for year, month in months:
        title = f"{calendar.month_name[month]} {year}"
        if year == now.year and month == now.month:
            header_color = C_HEADER_CUR
        else:
            header_color = C_HEADER_OTH

        grid_headers.append(f"{header_color}{title.center(20)}{RESET}")

        grid_days.append(f"{C_DAYS_LABEL}Mo Tu We Th Fr Sa Su{RESET}")

        month_weeks = cal.monthdayscalendar(year, month)
        while len(month_weeks) < 6:
            month_weeks.append([0] * 7)

        formatted_weeks = []
        for week in month_weeks:
            week_str = []
            for i, day in enumerate(week):
                if day == 0:
                    week_str.append("  ")
                    continue

                current_date = datetime.date(year, month, day)
                s_day = f"{day:>2}"

                if current_date < now:
                    week_str.append(f"{C_PAST}{s_day}{RESET}")
                elif current_date == now:
                    week_str.append(f"{C_TODAY_BG}{s_day}{RESET}")
                else:
                    is_weekend = i >= 5
                    if is_weekend:
                        week_str.append(f"{C_WEEKEND}{s_day}{RESET}")
                    else:
                        week_str.append(s_day)

            formatted_weeks.append(" ".join(week_str))
        grid_weeks.append(formatted_weeks)

    print("  ".join(grid_headers))
    print("  ".join(grid_days))

    for i in range(6):
        row = [grid_weeks[0][i], grid_weeks[1][i], grid_weeks[2][i]]
        print("  ".join(row))
