// .amp/health-check.ts
import { execute } from '@sourcegraph/amp-sdk';

async function checkCodebaseHealth() {
  console.log('🔍 Analyzing codebase against AGENT.md principles...\n');
  
  for await (const msg of execute({
    prompt: `Analyze this Bevy game codebase for potential concerns:

1. Read AGENT.md for our core principles
2. Check for violations of "Simplicity First" - any tangled/interweaved code?
3. Validate mesh-collider consistency (see AGENT.md section)
4. Check if physics systems follow the Dynamic Arcade Physics pattern
5. Look for any git safety violations (force pushes, uncommitted secrets)
6. Verify pre-commit rules are being followed

Report any concerns with file locations and severity (Critical/Warning/Info).`,
    
    options: {
      cwd: process.cwd(),
      // Safe: only allows Read, Grep, glob, finder - NO writes
      dangerouslyAllowAll: false
    }
  })) {
    // Handle all message types properly
    if (msg.type === 'assistant' && msg.content) {
      process.stdout.write(msg.content);
    } else if (msg.type === 'result') {
      console.log('\n\n✅ Health check complete');
      console.log('Result:', msg.result);
    }
  }
}

checkCodebaseHealth().catch(console.error);
