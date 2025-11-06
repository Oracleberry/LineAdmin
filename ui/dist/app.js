// Check if we're running in Tauri
const _isTauri = window.__TAURI__ !== undefined;

// Tauri API wrapper
const invoke = _isTauri ? window.__TAURI__.core.invoke : async (cmd, args) => {
    console.error('Tauri not available');
    return null;
};

// Tab switching
console.log('Initializing tab buttons...');
const tabButtons = document.querySelectorAll('.tab-button');
console.log('Found tab buttons:', tabButtons.length);

tabButtons.forEach(button => {
    button.addEventListener('click', () => {
        const tabName = button.dataset.tab;
        console.log('Tab clicked:', tabName);

        // Update button states
        document.querySelectorAll('.tab-button').forEach(b => b.classList.remove('active'));
        button.classList.add('active');

        // Update content states
        document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));
        const targetTab = document.getElementById(`${tabName}-tab`);
        console.log('Target tab element:', targetTab);
        if (targetTab) {
            targetTab.classList.add('active');
        }

        // Load data for the active tab
        switch(tabName) {
            case 'dashboard':
                loadDashboard();
                break;
            case 'users':
                loadUsers();
                break;
            case 'messages':
                loadMessages();
                break;
            case 'scheduled':
                loadScheduledMessages();
                loadUsersForSelect();
                break;
            case 'settings':
                loadSettings();
                break;
        }
    });
});

// Load users
async function loadUsers() {
    const container = document.getElementById('users-list');
    container.innerHTML = '<p class="loading">データを読み込み中...</p>';

    try {
        const users = await invoke('get_users');

        if (!users || users.length === 0) {
            container.innerHTML = '<p class="loading">登録ユーザーがいません</p>';
            return;
        }

        container.innerHTML = users.map(user => `
            <div class="data-item">
                <p><strong>LINE ID:</strong> ${user.line_user_id}</p>
                <p><strong>表示名:</strong> ${user.display_name || '未設定'}</p>
                <p><strong>登録日:</strong> ${formatDate(user.created_at)}</p>
            </div>
        `).join('');
    } catch (error) {
        container.innerHTML = `<p class="error">エラー: ${error}</p>`;
    }
}

// Load messages
async function loadMessages() {
    const container = document.getElementById('messages-list');
    const limit = parseInt(document.getElementById('message-limit').value);
    container.innerHTML = '<p class="loading">データを読み込み中...</p>';

    try {
        const messages = await invoke('get_messages', { limit });

        if (!messages || messages.length === 0) {
            container.innerHTML = '<p class="loading">メッセージがありません</p>';
            return;
        }

        container.innerHTML = messages.map(msg => `
            <div class="data-item">
                <p><strong>ユーザーID:</strong> ${msg.line_user_id}</p>
                <p><strong>タイプ:</strong> <span class="message-type">${msg.message_type}</span></p>
                ${msg.message_text ? `<p><strong>内容:</strong> ${msg.message_text}</p>` : ''}
                <p><strong>受信日時:</strong> ${formatDate(msg.timestamp)}</p>
            </div>
        `).join('');
    } catch (error) {
        container.innerHTML = `<p class="error">エラー: ${error}</p>`;
    }
}

// Load scheduled messages
async function loadScheduledMessages() {
    const container = document.getElementById('scheduled-list');
    container.innerHTML = '<p class="loading">データを読み込み中...</p>';

    try {
        const messages = await invoke('get_scheduled_messages');

        if (!messages || messages.length === 0) {
            container.innerHTML = '<p class="loading">スケジュール登録がありません</p>';
            return;
        }

        container.innerHTML = messages.map(msg => `
            <div class="data-item">
                <p><strong>配信先:</strong> ${msg.line_user_id || '全ユーザー'}</p>
                <p><strong>メッセージ:</strong> ${msg.message_text}</p>
                <p><strong>配信予定:</strong> ${formatDate(msg.schedule_time)}</p>
                <p><strong>ステータス:</strong> <span class="status-${msg.status}">${getStatusText(msg.status)}</span></p>
            </div>
        `).join('');
    } catch (error) {
        container.innerHTML = `<p class="error">エラー: ${error}</p>`;
    }
}

// Load users for select dropdown
async function loadUsersForSelect() {
    const select = document.getElementById('scheduled-user-id');

    try {
        const users = await invoke('get_users');

        // Keep the "All users" option and add user options
        const userOptions = users.map(user =>
            `<option value="${user.line_user_id}">${user.display_name || user.line_user_id}</option>`
        ).join('');

        select.innerHTML = '<option value="">全ユーザー（ブロードキャスト）</option>' + userOptions;
    } catch (error) {
        console.error('Failed to load users:', error);
    }
}

// Create scheduled message
async function createScheduledMessage(event) {
    event.preventDefault();

    const userId = document.getElementById('scheduled-user-id').value || null;
    const text = document.getElementById('scheduled-text').value;
    const time = document.getElementById('scheduled-time').value;

    // Convert to ISO 8601 format
    const scheduleTime = new Date(time).toISOString();

    try {
        await invoke('create_scheduled_message', {
            lineUserId: userId,
            messageText: text,
            scheduleTime: scheduleTime,
            cronExpression: null
        });

        alert('スケジュールを登録しました');
        document.getElementById('scheduled-form').reset();
        loadScheduledMessages();
    } catch (error) {
        alert(`エラー: ${error}`);
    }
}

// Load settings
async function loadSettings() {
    try {
        // Load LINE settings
        const lineToken = await invoke('get_setting', { key: 'line_channel_access_token' });
        const lineSecret = await invoke('get_setting', { key: 'line_channel_secret' });
        const lineNotifyToken = await invoke('get_setting', { key: 'line_notify_token' });
        const slackWebhook = await invoke('get_setting', { key: 'slack_webhook_url' });

        if (lineToken) document.getElementById('line-channel-token').value = lineToken;
        if (lineSecret) document.getElementById('line-channel-secret').value = lineSecret;
        if (lineNotifyToken) document.getElementById('line-notify-token').value = lineNotifyToken;
        if (slackWebhook) document.getElementById('slack-webhook-url').value = slackWebhook;
    } catch (error) {
        console.error('Failed to load settings:', error);
    }
}

// Save LINE settings
async function saveLineSetting(event) {
    event.preventDefault();

    const token = document.getElementById('line-channel-token').value;
    const secret = document.getElementById('line-channel-secret').value;

    try {
        await invoke('set_setting', {
            key: 'line_channel_access_token',
            value: token,
            description: 'LINE Channel Access Token'
        });

        await invoke('set_setting', {
            key: 'line_channel_secret',
            value: secret,
            description: 'LINE Channel Secret'
        });

        alert('LINE設定を保存しました');
    } catch (error) {
        alert(`エラー: ${error}`);
    }
}

// Save notification settings
async function saveNotificationSettings(event) {
    event.preventDefault();

    const lineNotifyToken = document.getElementById('line-notify-token').value;
    const slackWebhook = document.getElementById('slack-webhook-url').value;

    try {
        await invoke('set_setting', {
            key: 'line_notify_token',
            value: lineNotifyToken,
            description: 'LINE Notify Token'
        });

        await invoke('set_setting', {
            key: 'slack_webhook_url',
            value: slackWebhook,
            description: 'Slack Webhook URL'
        });

        alert('通知設定を保存しました');
    } catch (error) {
        alert(`エラー: ${error}`);
    }
}

// Utility functions
function formatDate(dateString) {
    if (!dateString) return '-';
    const date = new Date(dateString);
    return date.toLocaleString('ja-JP');
}

function getStatusText(status) {
    const statusMap = {
        'pending': '配信待ち',
        'sent': '配信完了',
        'failed': '配信失敗',
        'cancelled': 'キャンセル'
    };
    return statusMap[status] || status;
}

// External integrations
async function syncToNotion() {
    try {
        const result = await invoke('sync_to_notion');
        alert(result);
    } catch (error) {
        alert('エラー: ' + error);
    }
}

async function syncToAirtable() {
    try {
        const result = await invoke('sync_to_airtable');
        alert(result);
    } catch (error) {
        alert('エラー: ' + error);
    }
}

async function syncToGoogleSheets() {
    try {
        const result = await invoke('sync_to_google_sheets');
        alert(result);
    } catch (error) {
        alert('エラー: ' + error);
    }
}

// Dashboard functions
async function loadDashboard() {
    try {
        const stats = await invoke('get_dashboard_stats');

        // Update stat cards
        document.getElementById('total-users').textContent = stats.total_users.toLocaleString();
        document.getElementById('total-messages').textContent = stats.total_messages.toLocaleString();
        document.getElementById('messages-today').textContent = stats.messages_today.toLocaleString();
        document.getElementById('new-users-week').textContent = stats.new_users_this_week.toLocaleString();
        document.getElementById('pending-messages').textContent = stats.pending_scheduled_messages.toLocaleString();
        document.getElementById('upcoming-events').textContent = stats.upcoming_calendar_events.toLocaleString();

        // Render message types chart
        renderMessageTypesChart(stats.message_types);

        // Render hourly activity chart
        renderHourlyActivityChart(stats.hourly_activity);

    } catch (error) {
        console.error('Failed to load dashboard:', error);
    }
}

function renderMessageTypesChart(messageTypes) {
    const container = document.getElementById('message-types-chart');
    container.innerHTML = '';

    if (!messageTypes || messageTypes.length === 0) {
        container.innerHTML = '<p style="text-align: center; color: #999;">データがありません</p>';
        return;
    }

    const maxCount = Math.max(...messageTypes.map(m => m.count));

    messageTypes.forEach(type => {
        const bar = document.createElement('div');
        bar.className = 'bar';
        const heightPercent = (type.count / maxCount) * 100;
        bar.style.height = heightPercent + '%';

        const label = document.createElement('div');
        label.className = 'bar-label';
        label.textContent = type.message_type;

        const value = document.createElement('div');
        value.className = 'bar-value';
        value.textContent = type.count;

        bar.appendChild(label);
        bar.appendChild(value);
        container.appendChild(bar);
    });
}

function renderHourlyActivityChart(hourlyActivity) {
    const container = document.getElementById('hourly-activity-chart');
    container.innerHTML = '';

    if (!hourlyActivity || hourlyActivity.length === 0) {
        container.innerHTML = '<p style="text-align: center; color: #999;">データがありません</p>';
        return;
    }

    const maxCount = Math.max(...hourlyActivity.map(h => h.count), 1);

    // Fill in all 24 hours
    for (let hour = 0; hour < 24; hour++) {
        const data = hourlyActivity.find(h => h.hour === hour);
        const count = data ? data.count : 0;

        const bar = document.createElement('div');
        bar.className = 'bar';
        const heightPercent = maxCount > 0 ? (count / maxCount) * 100 : 0;
        bar.style.height = Math.max(heightPercent, 2) + '%';

        const label = document.createElement('div');
        label.className = 'bar-label';
        label.textContent = hour + '時';

        if (count > 0) {
            const value = document.createElement('div');
            value.className = 'bar-value';
            value.textContent = count;
            bar.appendChild(value);
        }

        bar.appendChild(label);
        container.appendChild(bar);
    }
}

// Initialize: Load dashboard on startup
// Since the script is at the end of body, DOM is already ready
loadDashboard();
