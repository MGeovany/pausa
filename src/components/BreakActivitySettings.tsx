import { useState, useEffect } from 'react';
import { tauriCommands } from '../lib/tauri';
import { breakActivityManager, activityCompletionTracker } from '../lib/breakActivities';
import type { BreakActivity } from '../types';

interface BreakActivitySettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

interface ActivityFormData {
  title: string;
  description: string;
  checklist: string[];
}

export function BreakActivitySettings({ isOpen, onClose }: BreakActivitySettingsProps) {
  const [activities, setActivities] = useState<BreakActivity[]>([]);
  const [isEditing, setIsEditing] = useState(false);
  const [editingActivity, setEditingActivity] = useState<BreakActivity | null>(null);
  const [formData, setFormData] = useState<ActivityFormData>({
    title: '',
    description: '',
    checklist: ['']
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  // Load activities when component opens
  useEffect(() => {
    if (isOpen) {
      loadActivities();
    }
  }, [isOpen]);

  const loadActivities = async () => {
    try {
      setLoading(true);
      // Get both default and custom activities
      const customActivities = await tauriCommands.getCustomActivities();
      const allActivities = breakActivityManager.getAllActivities();
      
      // Merge custom activities with defaults
      const mergedActivities = [...allActivities];
      customActivities.forEach(custom => {
        const existingIndex = mergedActivities.findIndex(a => a.title === custom.title);
        if (existingIndex >= 0) {
          mergedActivities[existingIndex] = custom;
        } else {
          mergedActivities.push(custom);
        }
      });
      
      setActivities(mergedActivities);
    } catch (err) {
      setError('Failed to load activities');
      console.error('Failed to load activities:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleAddActivity = () => {
    setIsEditing(true);
    setEditingActivity(null);
    setFormData({
      title: '',
      description: '',
      checklist: ['']
    });
  };

  const handleEditActivity = (activity: BreakActivity) => {
    setIsEditing(true);
    setEditingActivity(activity);
    setFormData({
      title: activity.title,
      description: activity.description,
      checklist: [...activity.checklist]
    });
  };

  const handleSaveActivity = async () => {
    if (!formData.title.trim() || !formData.description.trim()) {
      setError('Title and description are required');
      return;
    }

    const filteredChecklist = formData.checklist.filter(item => item.trim());
    if (filteredChecklist.length === 0) {
      setError('At least one checklist item is required');
      return;
    }

    const activity: BreakActivity = {
      title: formData.title.trim(),
      description: formData.description.trim(),
      checklist: filteredChecklist
    };

    try {
      setLoading(true);
      
      if (editingActivity) {
        // Update existing activity
        await tauriCommands.updateActivity(editingActivity.title, activity);
        breakActivityManager.updateActivity(editingActivity.title, activity);
      } else {
        // Add new activity
        await tauriCommands.addCustomActivity(activity);
        breakActivityManager.addCustomActivity(activity);
      }
      
      await loadActivities();
      setIsEditing(false);
      setError('');
    } catch (err) {
      setError('Failed to save activity');
      console.error('Failed to save activity:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteActivity = async (title: string) => {
    if (!confirm(`Are you sure you want to delete "${title}"?`)) return;

    try {
      setLoading(true);
      await tauriCommands.removeActivity(title);
      breakActivityManager.removeActivity(title);
      await loadActivities();
    } catch (err) {
      setError('Failed to delete activity');
      console.error('Failed to delete activity:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleChecklistItemChange = (index: number, value: string) => {
    const newChecklist = [...formData.checklist];
    newChecklist[index] = value;
    setFormData({ ...formData, checklist: newChecklist });
  };

  const addChecklistItem = () => {
    setFormData({
      ...formData,
      checklist: [...formData.checklist, '']
    });
  };

  const removeChecklistItem = (index: number) => {
    const newChecklist = formData.checklist.filter((_, i) => i !== index);
    setFormData({ ...formData, checklist: newChecklist });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4">
      <div className="bg-white rounded-2xl max-w-4xl w-full max-h-[90vh] overflow-hidden">
        <div className="p-6 border-b border-gray-200">
          <div className="flex items-center justify-between">
            <h2 className="text-2xl font-semibold text-gray-900">Break Activities</h2>
            <button
              onClick={onClose}
              className="p-2 hover:bg-gray-100 rounded-lg transition-colors"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <div className="p-6 overflow-y-auto max-h-[calc(90vh-140px)]">
          {error && (
            <div className="mb-4 p-3 bg-red-100 border border-red-300 rounded-lg">
              <p className="text-red-700 text-sm">{error}</p>
            </div>
          )}

          {!isEditing ? (
            // Activity list view
            <div>
              <div className="flex justify-between items-center mb-6">
                <p className="text-gray-600">
                  Manage your break activities. These will be randomly selected during breaks.
                </p>
                <button
                  onClick={handleAddActivity}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
                >
                  Add Activity
                </button>
              </div>

              <div className="grid gap-4">
                {activities.map((activity, index) => {
                  const completionRate = activityCompletionTracker.getActivityCompletionRate(activity.title);
                  
                  return (
                    <div key={index} className="border border-gray-200 rounded-lg p-4">
                      <div className="flex justify-between items-start mb-2">
                        <h3 className="text-lg font-medium text-gray-900">{activity.title}</h3>
                        <div className="flex space-x-2">
                          <button
                            onClick={() => handleEditActivity(activity)}
                            className="p-1 hover:bg-gray-100 rounded text-gray-500 hover:text-gray-700"
                          >
                            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                            </svg>
                          </button>
                          <button
                            onClick={() => handleDeleteActivity(activity.title)}
                            className="p-1 hover:bg-gray-100 rounded text-gray-500 hover:text-red-600"
                          >
                            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                          </button>
                        </div>
                      </div>
                      
                      <p className="text-gray-600 text-sm mb-3">{activity.description}</p>
                      
                      <div className="mb-3">
                        <h4 className="text-sm font-medium text-gray-700 mb-2">Checklist:</h4>
                        <ul className="text-sm text-gray-600 space-y-1">
                          {activity.checklist.map((item, itemIndex) => (
                            <li key={itemIndex} className="flex items-center">
                              <span className="w-2 h-2 bg-gray-300 rounded-full mr-2"></span>
                              {item}
                            </li>
                          ))}
                        </ul>
                      </div>
                      
                      {completionRate > 0 && (
                        <div className="text-xs text-gray-500">
                          Completion rate: {completionRate.toFixed(1)}%
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          ) : (
            // Activity form view
            <div>
              <div className="mb-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">
                  {editingActivity ? 'Edit Activity' : 'Add New Activity'}
                </h3>
                
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      Activity Title
                    </label>
                    <input
                      type="text"
                      value={formData.title}
                      onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                      placeholder="e.g., Stretch and Move"
                    />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      Description
                    </label>
                    <textarea
                      value={formData.description}
                      onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                      rows={3}
                      className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                      placeholder="Describe what this activity is about..."
                    />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      Checklist Items
                    </label>
                    <div className="space-y-2">
                      {formData.checklist.map((item, index) => (
                        <div key={index} className="flex items-center space-x-2">
                          <input
                            type="text"
                            value={item}
                            onChange={(e) => handleChecklistItemChange(index, e.target.value)}
                            className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            placeholder={`Checklist item ${index + 1}`}
                          />
                          {formData.checklist.length > 1 && (
                            <button
                              onClick={() => removeChecklistItem(index)}
                              className="p-2 text-red-500 hover:bg-red-50 rounded-lg"
                            >
                              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                              </svg>
                            </button>
                          )}
                        </div>
                      ))}
                      <button
                        onClick={addChecklistItem}
                        className="text-blue-600 hover:text-blue-700 text-sm font-medium"
                      >
                        + Add checklist item
                      </button>
                    </div>
                  </div>
                </div>
              </div>
              
              <div className="flex justify-end space-x-3">
                <button
                  onClick={() => setIsEditing(false)}
                  className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSaveActivity}
                  disabled={loading}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-lg transition-colors"
                >
                  {loading ? 'Saving...' : 'Save Activity'}
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
