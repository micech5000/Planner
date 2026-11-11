const {chromium}=require('C:/Users/exzit/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const {pathToFileURL}=require('url');
(async()=>{
 const browser=await chromium.launch({headless:true,channel:'msedge'});
 const page=await browser.newPage({viewport:{width:1100,height:1200}});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(pathToFileURL('C:/Users/exzit/Desktop/StudyPlanner/preview-check.html').href);
 const frame=page.frameLocator('iframe');
 await frame.locator('#planner-concept').waitFor();
 await frame.locator('[data-task="algebra"]').first().check();
 if(!await frame.locator('[data-task="algebra"]').last().isChecked())throw Error('Widget checkbox did not synchronize');
 await frame.locator('.p-nav [data-screen="sleep"]').click();
 await frame.locator('[data-setting="sleep"]').fill('9');
 if(await frame.locator('[data-panel="sleep"] [data-finish]').textContent()!=='21:55')throw Error('Sleep calculation failed');
 await frame.locator('[data-setting="sleep"]').fill('8');
 await frame.locator('.p-nav [data-screen="today"]').click();
 await frame.locator('[data-task="algebra"]').first().uncheck();
 for(const width of [1024,736,320]){
  await page.setViewportSize({width,height:1500});
  await frame.locator('#planner-concept').screenshot({path:`C:/Users/exzit/Desktop/StudyPlanner/preview-${width}.png`});
  const overflow=await frame.locator('#planner-concept').evaluate(el=>el.scrollWidth>el.clientWidth+1);
  if(overflow)throw Error('Horizontal overflow at '+width);
 }
 console.log(JSON.stringify({checkboxSync:true,sleepCalculation:true,checkedWidths:[1024,736,320],errors}));
 await browser.close();if(errors.length)process.exitCode=1;
})().catch(e=>{console.error(e);process.exitCode=1});
