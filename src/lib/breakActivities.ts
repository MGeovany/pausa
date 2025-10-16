import type { BreakActivity } from '../types';

// Default break activities database
const DEFAULT_ACTIVITIES: BreakActivity[] = [
  {
    title: "Hydrate & Refresh",
    description: "Take care of your body's basic needs and refresh your mind.",
    checklist: [
      "Drink a full glass of water",
      "Splash cool water on your face",
      "Take 5 deep breaths",
      "Stretch your neck and shoulders"
    ]
  },
  {
    title: "Move Your Body",
    description: "Get your blood flowing and relieve tension from sitting.",
    checklist: [
      "Stand up and walk around",
      "Do 10 jumping jacks or squats",
      "Stretch your arms above your head",
      "Roll your shoulders backward 5 times"
    ]
  },
  {
    title: "Rest Your Eyes",
    description: "Give your eyes a break from screen strain and refocus.",
    checklist: [
      "Look out a window at something far away",
      "Close your eyes for 30 seconds",
      "Blink slowly 10 times",
      "Massage your temples gently"
    ]
  },
  {
    title: "Mindful Moment",
    description: "Center yourself and practice mindfulness.",
    checklist: [
      "Sit comfortably and close your eyes",
      "Focus on your breathing for 1 minute",
      "Notice 3 things you can hear",
      "Set an intention for your next focus session"
    ]
  },
  {
    title: "Quick Tidy",
    description: "Organize your space for better focus when you return.",
    checklist: [
      "Clear your desk of unnecessary items",
      "Put away 3 things that are out of place",
      "Empty your trash or recycling",
      "Adjust your lighting and chair position"
    ]
  },
  {
    title: "Social Connection",
    description: "Connect with others to boost your mood and energy.",
    checklist: [
      "Send a quick message to a friend or family member",
      "Check in with a colleague",
      "Give someone a genuine compliment",
      "Smile at yourself in the mirror"
    ]
  },
  {
    title: "Fresh Air Break",
    description: "Step outside or get some fresh air to rejuvenate.",
    checklist: [
      "Open a window or step outside",
      "Take 5 deep breaths of fresh air",
      "Feel the temperature on your skin",
      "Look up at the sky for a moment"
    ]
  },
  {
    title: "Fuel Your Body",
    description: "Nourish yourself with healthy snacks and hydration.",
    checklist: [
      "Eat a piece of fruit or healthy snack",
      "Drink water or herbal tea",
      "Avoid sugary or caffeinated drinks",
      "Chew slowly and mindfully"
    ]
  }
];

// Activity rotation and selection logic
export class BreakActivityManager {
  private activities: BreakActivity[];
  private lastUsedIndexes: number[] = [];
  private maxHistorySize = 3; // Avoid repeating last 3 activities

  constructor(customActivities?: BreakActivity[]) {
    this.activities = customActivities || [...DEFAULT_ACTIVITIES];
  }

  /**
   * Get a random activity that hasn't been used recently
   */
  getRandomActivity(): BreakActivity {
    const availableIndexes = this.activities
      .map((_, index) => index)
      .filter(index => !this.lastUsedIndexes.includes(index));

    // If all activities have been used recently, reset the history
    if (availableIndexes.length === 0) {
      this.lastUsedIndexes = [];
      return this.activities[Math.floor(Math.random() * this.activities.length)];
    }

    const randomIndex = availableIndexes[Math.floor(Math.random() * availableIndexes.length)];
    this.addToHistory(randomIndex);

    return this.activities[randomIndex];
  }

  /**
   * Get an activity suitable for the break type and duration
   */
  getActivityForBreak(breakType: 'short' | 'long', duration: number): BreakActivity {
    // For short breaks (< 10 minutes), prefer simpler activities
    if (breakType === 'short' || duration < 600) {
      const shortBreakActivities = this.activities.filter(activity =>
        activity.checklist.length <= 4 &&
        (activity.title.includes('Hydrate') ||
          activity.title.includes('Eyes') ||
          activity.title.includes('Move') ||
          activity.title.includes('Mindful'))
      );

      if (shortBreakActivities.length > 0) {
        const availableActivities = shortBreakActivities.filter(activity => {
          const index = this.activities.indexOf(activity);
          return !this.lastUsedIndexes.includes(index);
        });

        if (availableActivities.length > 0) {
          const selected = availableActivities[Math.floor(Math.random() * availableActivities.length)];
          this.addToHistory(this.activities.indexOf(selected));
          return selected;
        }
      }
    }

    // For long breaks, any activity is suitable
    return this.getRandomActivity();
  }

  /**
   * Get activity by title (for user preferences)
   */
  getActivityByTitle(title: string): BreakActivity | null {
    return this.activities.find(activity => activity.title === title) || null;
  }

  /**
   * Add a custom activity
   */
  addCustomActivity(activity: BreakActivity): void {
    this.activities.push(activity);
  }

  /**
   * Remove an activity by title
   */
  removeActivity(title: string): boolean {
    const index = this.activities.findIndex(activity => activity.title === title);
    if (index !== -1) {
      this.activities.splice(index, 1);
      // Clean up history to avoid invalid indexes
      this.lastUsedIndexes = this.lastUsedIndexes.filter(i => i < this.activities.length);
      return true;
    }
    return false;
  }

  /**
   * Get all available activities
   */
  getAllActivities(): BreakActivity[] {
    return [...this.activities];
  }

  /**
   * Update an existing activity
   */
  updateActivity(oldTitle: string, newActivity: BreakActivity): boolean {
    const index = this.activities.findIndex(activity => activity.title === oldTitle);
    if (index !== -1) {
      this.activities[index] = newActivity;
      return true;
    }
    return false;
  }

  private addToHistory(index: number): void {
    this.lastUsedIndexes.push(index);
    if (this.lastUsedIndexes.length > this.maxHistorySize) {
      this.lastUsedIndexes.shift();
    }
  }
}

// Singleton instance for the app
export const breakActivityManager = new BreakActivityManager();

// Utility functions for activity completion tracking
export interface ActivityCompletion {
  activityTitle: string;
  completedItems: boolean[];
  completedAt: Date;
  breakSessionId: string;
}

export class ActivityCompletionTracker {
  private completions: ActivityCompletion[] = [];

  /**
   * Record activity completion
   */
  recordCompletion(
    activityTitle: string,
    completedItems: boolean[],
    breakSessionId: string
  ): void {
    const completion: ActivityCompletion = {
      activityTitle,
      completedItems,
      completedAt: new Date(),
      breakSessionId
    };

    this.completions.push(completion);

    // Keep only last 100 completions to avoid memory issues
    if (this.completions.length > 100) {
      this.completions = this.completions.slice(-100);
    }
  }

  /**
   * Get completion rate for an activity
   */
  getActivityCompletionRate(activityTitle: string): number {
    const activityCompletions = this.completions.filter(
      completion => completion.activityTitle === activityTitle
    );

    if (activityCompletions.length === 0) return 0;

    const totalItems = activityCompletions.reduce(
      (sum, completion) => sum + completion.completedItems.length, 0
    );
    const completedItems = activityCompletions.reduce(
      (sum, completion) => sum + completion.completedItems.filter(Boolean).length, 0
    );

    return totalItems > 0 ? (completedItems / totalItems) * 100 : 0;
  }

  /**
   * Get recent completions
   */
  getRecentCompletions(days: number = 7): ActivityCompletion[] {
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - days);

    return this.completions.filter(
      completion => completion.completedAt >= cutoffDate
    );
  }

  /**
   * Get most and least completed activities
   */
  getActivityStats(): { mostCompleted: string; leastCompleted: string } {
    const activityRates = new Map<string, number>();

    // Get unique activity titles
    const uniqueActivities = [...new Set(this.completions.map(c => c.activityTitle))];

    uniqueActivities.forEach(title => {
      activityRates.set(title, this.getActivityCompletionRate(title));
    });

    let mostCompleted = '';
    let leastCompleted = '';
    let highestRate = -1;
    let lowestRate = 101;

    activityRates.forEach((rate, title) => {
      if (rate > highestRate) {
        highestRate = rate;
        mostCompleted = title;
      }
      if (rate < lowestRate) {
        lowestRate = rate;
        leastCompleted = title;
      }
    });

    return { mostCompleted, leastCompleted };
  }
}

// Singleton instance for completion tracking
export const activityCompletionTracker = new ActivityCompletionTracker();
