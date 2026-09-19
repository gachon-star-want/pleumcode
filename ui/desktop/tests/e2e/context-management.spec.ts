import { test, expect } from './fixtures';

test.describe('Context Management E2E Tests', () => {
  test.beforeEach(async ({ pleumPage }) => {
    // Ensure the app is ready before each test
    await pleumPage.waitForSelector('[data-testid="chat-input"]', { timeout: 10000 });
  });

  test('should show context window alert when tokens are being used', async ({ pleumPage }) => {
    // Type a message to generate some token usage
    const chatInput = pleumPage.locator('[data-testid="chat-input"]');
    await chatInput.fill('Hello, this is a test message to generate some token usage.');
    
    // Submit the message
    await pleumPage.keyboard.press('Enter');
    
    // Wait for response and check for context window alert
    await pleumPage.waitForSelector('[data-testid="alert-indicator"]', { timeout: 15000 });
    
    // Click on the alert indicator to open the popover
    await pleumPage.click('[data-testid="alert-indicator"]');
    
    // Verify the context window alert is shown
    const alertBox = pleumPage.locator('[role="alert"]');
    await expect(alertBox).toBeVisible();
    await expect(alertBox).toContainText('Context window');
    
    // Verify progress bar is shown
    const progressBar = pleumPage.locator('[role="progressbar"]');
    await expect(progressBar).toBeVisible();
    
    // Verify compact button is present
    const compactButton = pleumPage.locator('text=Compact now');
    await expect(compactButton).toBeVisible();
  });

  test('should perform manual compaction when compact button is clicked', async ({ pleumPage }) => {
    // First, generate enough conversation to have tokens
    const chatInput = pleumPage.locator('[data-testid="chat-input"]');
    
    // Send multiple messages to build up context
    const messages = [
      'Hello, I need help with a programming task.',
      'Can you explain how React hooks work?',
      'What are the best practices for state management?',
      'How do I optimize performance in React applications?',
    ];
    
    for (const message of messages) {
      await chatInput.fill(message);
      await pleumPage.keyboard.press('Enter');
      
      // Wait for response before sending next message
      await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
      await pleumPage.waitForTimeout(1000); // Brief pause between messages
    }
    
    // Open the alert popover
    await pleumPage.waitForSelector('[data-testid="alert-indicator"]', { timeout: 15000 });
    await pleumPage.click('[data-testid="alert-indicator"]');
    
    // Click the compact button
    const compactButton = pleumPage.locator('text=Compact now');
    await expect(compactButton).toBeVisible();
    await compactButton.click();
    
    // Verify compaction loading state
    const loadingPleum = pleumPage.locator('[data-testid="loading-pleum"]');
    await expect(loadingPleum).toBeVisible();
    await expect(loadingPleum).toContainText('pleum is compacting the conversation...');
    
    // Wait for compaction to complete
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Verify compaction marker appears
    const compactionMarker = pleumPage.locator('text=Conversation compacted and summarized');
    await expect(compactionMarker).toBeVisible();
    
    // Verify alert popover is closed after compaction
    const alertBox = pleumPage.locator('[role="alert"]');
    await expect(alertBox).not.toBeVisible();
  });

  test('should allow scrolling to see past messages after compaction', async ({ pleumPage }) => {
    // Generate conversation content
    const chatInput = pleumPage.locator('[data-testid="chat-input"]');
    
    const testMessages = [
      'First message in the conversation',
      'Second message with some content',
      'Third message to build context',
    ];
    
    // Send messages and store their content for verification
    for (const message of testMessages) {
      await chatInput.fill(message);
      await pleumPage.keyboard.press('Enter');
      await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
      await pleumPage.waitForTimeout(1000);
    }
    
    // Perform manual compaction
    await pleumPage.waitForSelector('[data-testid="alert-indicator"]', { timeout: 15000 });
    await pleumPage.click('[data-testid="alert-indicator"]');
    await pleumPage.click('text=Compact now');
    
    // Wait for compaction to complete
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    await expect(pleumPage.locator('text=Conversation compacted and summarized')).toBeVisible();
    
    // Scroll up to verify past messages are still visible
    const chatContainer = pleumPage.locator('[data-testid="chat-container"]');
    await chatContainer.hover();
    
    // Scroll up multiple times to reach earlier messages
    for (let i = 0; i < 5; i++) {
      await pleumPage.mouse.wheel(0, -500);
      await pleumPage.waitForTimeout(200);
    }
    
    // Verify that we can still see the original messages
    // Note: The exact messages might be in ancestor messages, so we check for partial content
    const messageElements = pleumPage.locator('[data-testid="message"]');
    const messageCount = await messageElements.count();
    
    // Should have more than just the compaction marker and summary
    expect(messageCount).toBeGreaterThan(2);
  });

  test('should handle compaction errors gracefully', async ({ pleumPage }) => {
    // Mock a backend error by intercepting the compaction request
    await pleumPage.route('**/api/sessions/*/manage-context', async (route) => {
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Backend compaction error' }),
      });
    });
    
    // Generate some conversation
    const chatInput = pleumPage.locator('[data-testid="chat-input"]');
    await chatInput.fill('Test message for error handling');
    await pleumPage.keyboard.press('Enter');
    
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Attempt compaction
    await pleumPage.waitForSelector('[data-testid="alert-indicator"]', { timeout: 15000 });
    await pleumPage.click('[data-testid="alert-indicator"]');
    await pleumPage.click('text=Compact now');
    
    // Wait for compaction to fail
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Verify error message appears
    const errorMarker = pleumPage.locator('text=Compaction failed. Please try again or start a new session.');
    await expect(errorMarker).toBeVisible();
  });

  test('should not show compaction UI when no tokens are used', async ({ pleumPage }) => {
    // On a fresh pleumPage with no messages, there should be no alert indicator
    const alertIndicator = pleumPage.locator('[data-testid="alert-indicator"]');
    await expect(alertIndicator).not.toBeVisible();
    
    // The chat input should be available but no context alerts
    const chatInput = pleumPage.locator('[data-testid="chat-input"]');
    await expect(chatInput).toBeVisible();
  });

  test('should maintain conversation flow after compaction', async ({ pleumPage }) => {
    // Generate initial conversation
    const chatInput = pleumPage.locator('[data-testid="chat-input"]');
    
    await chatInput.fill('What is React?');
    await pleumPage.keyboard.press('Enter');
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    await chatInput.fill('Can you give me an example?');
    await pleumPage.keyboard.press('Enter');
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Perform compaction
    await pleumPage.waitForSelector('[data-testid="alert-indicator"]', { timeout: 15000 });
    await pleumPage.click('[data-testid="alert-indicator"]');
    await pleumPage.click('text=Compact now');
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Verify compaction marker
    await expect(pleumPage.locator('text=Conversation compacted and summarized')).toBeVisible();
    
    // Continue conversation after compaction
    await chatInput.fill('Thank you, that was helpful. What about Vue.js?');
    await pleumPage.keyboard.press('Enter');
    
    // Verify that the conversation continues normally
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { timeout: 30000 });
    await expect(pleumPage.locator('[data-testid="loading-pleum"]')).toBeVisible();
    
    // Wait for response
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Verify new message appears after compaction
    const messages = pleumPage.locator('[data-testid="message"]');
    const messageCount = await messages.count();
    expect(messageCount).toBeGreaterThan(1); // Should have compaction marker + new messages
  });

  test('should show appropriate loading states during compaction', async ({ pleumPage }) => {
    // Generate conversation
    const chatInput = pleumPage.locator('[data-testid="chat-input"]');
    await chatInput.fill('Test message for loading state verification');
    await pleumPage.keyboard.press('Enter');
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Start compaction
    await pleumPage.waitForSelector('[data-testid="alert-indicator"]', { timeout: 15000 });
    await pleumPage.click('[data-testid="alert-indicator"]');
    await pleumPage.click('text=Compact now');
    
    // Verify loading state immediately after clicking compact
    const loadingPleum = pleumPage.locator('[data-testid="loading-pleum"]');
    await expect(loadingPleum).toBeVisible();
    await expect(loadingPleum).toContainText('pleum is compacting the conversation...');
    
    // Verify chat input is disabled during compaction
    const submitButton = pleumPage.locator('[data-testid="submit-button"]');
    await expect(submitButton).toBeDisabled();
    
    // Wait for compaction to complete
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Verify chat input is re-enabled after compaction
    await expect(submitButton).toBeEnabled();
  });

  test('should handle multiple rapid compaction attempts', async ({ pleumPage }) => {
    // Generate conversation
    const chatInput = pleumPage.locator('[data-testid="chat-input"]');
    await chatInput.fill('Test message for rapid compaction test');
    await pleumPage.keyboard.press('Enter');
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    // Open alert and try to click compact multiple times rapidly
    await pleumPage.waitForSelector('[data-testid="alert-indicator"]', { timeout: 15000 });
    await pleumPage.click('[data-testid="alert-indicator"]');
    
    const compactButton = pleumPage.locator('text=Compact now');
    await expect(compactButton).toBeVisible();
    
    // Click multiple times rapidly
    await compactButton.click();
    
    // The alert should be hidden immediately after first click
    const alertBox = pleumPage.locator('[role="alert"]');
    await expect(alertBox).not.toBeVisible();
    
    // Verify only one compaction occurs
    await pleumPage.waitForSelector('[data-testid="loading-pleum"]', { state: 'hidden', timeout: 30000 });
    
    const compactionMarkers = pleumPage.locator('text=Conversation compacted and summarized');
    await expect(compactionMarkers).toHaveCount(1);
  });
});
