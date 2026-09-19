# 코딩 하네스 설계 원칙

> 리서치 완료 2026-09-19 (워크플로우 `wf_45fe42c2-dc8`, 3개 주제 병렬 리서치 → 1개 종합). "코딩을 체계적이고
> 잘하는" 에이전트가 되기 위한 최신 논문/벤치마크/안전성 트렌드 조사 결과.

## 핵심 설계 원칙: 단일 오케스트레이션 프리미티브를 재사용한다

[`01-unique-features.md`](./01-unique-features.md)에서 확정한 pleumcode의 독창 기능이 "역할별로 다른 모델을
배정하는 멀티모델 서브에이전트 오케스트레이션"이다. 이 리서치가 요구하는 4가지 별개 메커니즘 —
(a) worker 위임, (b) adversarial reviewer/critic, (c) 리스크 태스크의 best-of-N 후보 생성,
(d) rollout 요약 distillation — 을 **각각 새로 만들지 않고 전부 이 하나의 서브에이전트 디스패치 위에 얹는다.**
오케스트레이터-워커 계약(worker는 좁은 브리프+격리 컨텍스트, 최종 결과+아티팩트 참조만 반환, 대화 히스토리
비공유)을 네 용도 모두의 공통 인터페이스로 삼는다.

**구현**: Goose의 recipe/sub-recipe + subagent(Tasks) 디스패치를 유일한 위임 채널로 고정. 호출 시 공통
파라미터로 `role`(worker/reviewer/critic/candidate), `model`(라우팅 슬러그 — `policy/<slug>` 등 게이트웨이
패스스루), `context_scope`만 다르게 넣고, 반환 스키마는 4용도 공통 `{result, changed_files, verdict?,
confidence?}`로 통일한다. 별도 리뷰어 파이프라인, 별도 best-of-N 러너를 각각 구현하지 않는다.

---

## 11개 하네스 원칙

### 1. Ground-truth 검증 루프 (3회 캡 후 에스컬레이션)

가장 검증된 단일 레버: 테스트/타입체크/린트/빌드를 하나의 boolean 신호로 압축 실행하고, 원문 실패 로그
(요약 아님)를 다음 턴에 그대로 피드백하며, 자체 수정 시도를 작은 고정 예산(약 3회)으로 제한한 뒤 실패하면
계속 도는 대신 사용자에게 넘긴다. Aider의 `max_reflections=3`, Claude Code Stop hook의 exit-code-2 게이트,
AlphaCodium의 flow engineering(GPT-4 pass@5를 CodeContests에서 19%→44%로 끌어올림, 모델 변경 없이)이
모두 이 원칙의 변주다.

**구현**: AGENTS.md/`.goosehints`에 프로젝트별 `verify_cmd`(테스트/린트/타입체크 조합)를 선언하는 컨벤션을
v1에 고정. goose-mcp의 developer extension(shell/text_editor 툴)이 파일 write 직후 `verify_cmd`를 자동
실행하도록 감싸는 얇은 wrapper(신규 MCP 확장 하나, core 수정 없음)를 추가하고, 실패 시 raw stdout/stderr를
다음 턴 컨텍스트에 그대로 주입. 실패-수정 사이클 카운터를 이 wrapper의 세션 상태에 하드코딩 3회로 캡.
`ponytail`: 범용 정책 엔진을 만들지 말고 3회 하드캡으로 시작, 실제로 부족하면 그때 늘림.

### 2. 리스크 티어링 best-of-N + execute-and-vote (트리서치는 만들지 않음)

모든 태스크에 병렬 샘플링을 쓰지 않는다. auth/결제/마이그레이션/동시성을 건드리거나 테스트 오라클이 없는
"리스키"한 경우에만 N(≤5~8)개 후보를 병렬 생성해 실행 결과로 투표(execute-and-vote)한다. LATS류 MCTS나
Scattered Forest Search 같은 트리서치는 v1 스코프에서 제외 — 디베이트 대비 다수결 투표만으로 대부분의
이득을 저비용으로 얻는다는 2026 연구(Debate or Vote, NeurIPS 2025 spotlight)와도 일치한다.

**구현**: 리스크 태그를 `pleumcode.md`에 경로 패턴(`**/auth/**`, `**/billing/**`, `**/migrations/**`)으로
선언해두고, 편집 대상 파일이 매칭되면 자동으로 N=5 후보 모드 진입. 원칙 1의 오케스트레이션 프리미티브로
5개 워커를 각자 git worktree에서 병렬 실행시키고, 각 워크트리에서 `verify_cmd`(원칙 1)를 돌려 통과 테스트
수가 가장 많은 후보를 채택 — 별도 스코어링/보상모델을 학습·호스팅하지 않는다.

### 3. Rollout → 구조화 요약 압축 후 재사용 (원본 트랜스크립트 전달 금지)

병렬(best-of-N)이든 순차(재시도)든, 이전 시도의 전체 대화 기록을 다음 시도/판정에 그대로 넘기지 않는다.
각 rollout을 "핵심 가설·진행상황·실패모드"만 남긴 구조화 요약으로 압축한 뒤에만 다음 컨텍스트에 주입한다
— 2026 연구("Scaling Test-Time Compute for Agentic Coding")에서 이 압축·선택 설계가 단순 반복 샘플링보다
효율의 핵심으로 확인됨(Claude-4.5-Opus + mini-SWE-agent, SWE-bench Verified 70.9%→77.6%).

**구현**: 원칙 1의 서브에이전트 반환 스키마를 `{hypothesis, changed_files, test_result, failure_mode}`로
강제 — 이게 곧 구조화 요약이라 별도 압축 파이프라인이 필요 없다. 순차 재시도 시 이전 시도의 `failure_mode`
필드만 다음 워커의 초기 컨텍스트에 주입하고 전체 히스토리는 버린다.

### 4. 두 계층 외부화 메모리 + "두 번 규칙" + 로드 예산 캡

실수를 다음 실행이 학습하도록 파일로 외부화하되, 노이즈 축적을 막는 두 규칙을 같이 넣는다: (1) 같은 실패가
**두 번째** 발생했을 때만 영구 규칙으로 기록(1회는 우연일 수 있음), (2) 메모리 파일은 세션 시작 시 로드되는
상한(줄 수/KB)을 두어 중요한 것이 상단에 오도록 강제. OpenHands의 always-loaded `repo.md` + 키워드 트리거
microagent 이원 구조, Anthropic의 "두 번 규칙"이 근거.

**구현**: Goose의 `.goosehints`(레포 루트, 항상 로드)를 tool-neutral 공개 문서(AGENTS.md)와 fork 전용
워크플로우 규칙(`pleumcode.md`)으로 분리하고 `pleumcode.md`가 AGENTS.md를 import하는 구조 — 새 파일
로더를 만들지 않고 기존 `.goosehints` 로딩 메커니즘 그대로 사용. "두 번 규칙"은 자동화하지 않고 팀 컨벤션
(PR 리뷰 체크리스트 한 줄)으로만 강제. 로드 예산은 "이 파일은 200줄/25KB를 넘기지 않는다"고 컨벤션 문서에
명시하는 선에서 시작(자동 truncate 로직은 실제로 파일이 커질 때까지 만들지 않음).

### 5. Adversarial review — frozen-artifact Reviewer + evidence-typed verdict

패치를 만든 에이전트가 스스로를 채점하지 않는다. 별도 컨텍스트의 Reviewer가 고정된(freeze) 결과물을
검사하고, 판정을 자유서술이 아니라 `AGREE`/`DISAGREE_EVIDENCE`(반드시 근거 코드 인용)/`DISAGREE_CONCERN`
3가지로 이산화해 rubber-stamping(false consensus)을 구조적으로 막는다. "Adversarial Review" 논문(arXiv
2608.18167, 2026-08) 기준 evidence-typed 제약이 F1을 0.457→0.533으로 끌어올린 핵심 요인이며, 3-role
프로토콜이 5-agent 베이스라인(MARS)보다 LiveCodeBench 87% vs 82%, SWE-bench Verified 75.2% vs 72.6%로
더 나은 결과를 냈다.

**구현**: 원칙의 서브에이전트 디스패치로 Reviewer를 별도 role로 호출(Main과 히스토리 비공유, diff+test
결과만 전달), 반환 스키마를 `{verdict: enum[AGREE, DISAGREE_EVIDENCE, DISAGREE_CONCERN], evidence?: string}`로
고정해 `DISAGREE_EVIDENCE`일 때 `evidence` 필드를 필수로 강제. `ponytail`: 3번째 역할(Critic-of-Reviewer)은
v1 스코프에서 제외 — 2-role만으로도 zero-shot 대비 유의미한 개선이 있고, false consensus가 실제로 관찰되면
그때 Critic 레이어 추가.

### 6. 신뢰도×가역성 2축 에스컬레이션

"항상 물어보기"도 "절대 안 물어보기"도 아닌, 계획의 자체 신뢰도와 행동의 가역성이라는 두 독립축으로
자동진행/승인대기를 가른다. 고신뢰+가역적 = 자동진행(비동기 상태 업데이트만), 저신뢰 또는 비가역적(prod
배포/스키마 삭제/force-push) = 반드시 차단 후 승인. Devin 2.1의 confidence-scored plan이 실증 사례
(고신뢰 "green" 플랜이 저신뢰 "red" 플랜보다 병합 PR로 이어질 확률이 약 2배).

**구현**: Goose에 이미 존재하는 `GOOSE_MODE=smart_approve`(기존 4모드 중 하나, [`04-fork-plan.md`](./04-fork-plan.md)
참고)를 신규로 만들지 않고 확장 — 현재 툴 이름 기반 휴리스틱에 (a) 에이전트가 계획 단계에서 emit하는
confidence 필드, (b) `pleumcode.md`에 선언된 비가역 명령 목록(rm, force-push, migrate, deploy)을 추가
입력으로 반영해 승인 여부를 결정. 새 승인 시스템을 처음부터 짜지 않는다.

### 7. ACI(Agent-Computer Interface) 설계 원칙

LLM을 사람용 셸의 사용자가 아니라 고유한 한계를 가진 별종 사용자로 취급한다: 파일은 윈도우 단위로만 노출
(bounded view), 네비게이션 툴은 좁게(find/search 한정), 편집 직후 린트를 가드레일로 강제, 히스토리는
압축해서 유지. SWE-agent 원 논문에서 이 설계만으로 GPT-4 Turbo가 당시 SOTA(SWE-bench Lite 18%)를 찍었고
통제실험으로 각 요소의 기여가 확인됨.

**구현**: goose-mcp developer extension의 text_editor 툴(view/str_replace/write)이 이미 windowed view에
가까운 구조이므로 그대로 활용하되, `.gooseignore`로 노출 범위를 프로젝트 관련 파일로 좁히고, write 직후
자동 린트(원칙 1 wrapper의 부분집합)를 편집 가드레일로 강제. 기존 "토큰 80% 도달 시 자동 컨텍스트 압축
(Smart Context Management)" 기능([`04-fork-plan.md`](./04-fork-plan.md) 기재)을 히스토리 압축 메커니즘으로
그대로 재사용 — 별도 압축기를 새로 만들지 않는다.

### 8. 조기 "완료" 선언 가로채기 + git/progress 기반 세션 복구

에이전트가 스스로 "끝났다"고 선언하려는 시점을 가로채 성공 기준(JSON feature-list, `passes:false` 필드 등
— 마크다운보다 모델 손상에 강함)을 별도로 재검사하고, 미충족이면 같은 프롬프트를 갱신된 컨텍스트로
재투입한다("Ralph Loop" 패턴). 세션 간 상태는 `progress.txt` + git 커밋 히스토리로 복구하고, 매 세션 종료
시 "머지 가능한 클린 상태"를 강제한다.

**구현**: 장시간 태스크용 recipe에 feature-list를 JSON(YAML 아님)으로 정의하고 각 항목에 `passes` 필드를
두어, 원칙 1의 `verify_cmd` 결과로만 `passes:true`를 세팅(에이전트 자기선언으로는 true 불가). Goose의 세션
JSONL 저장(`goose session resume`) 위에 얇게 `progress.txt` 컨벤션만 얹는다 — 새 상태저장 시스템을 만들지
않는다.

### 9. 검증 계층화 — 결정론적 체크 우선, 추론적 체크는 후순위

린트/타입체크/빌드/구조 테스트 같은 저비용·결정론적 체크를 항상 먼저 돌리고, LLM-judge나 adversarial
review 같은 고비용·추론적 체크는 그 다음에만 실행한다. 결정론적 체크로 걸러지는 실패에 굳이 비싼 리뷰어를
호출하지 않는 것이 원칙 1/5의 비용 효율을 지킨다.

**구현**: 원칙 1의 `verify_cmd` wrapper가 실행 순서를 강제: (1) typecheck/lint/build → (2) test → (3) 모두
통과한 것에 한해서만 원칙 5의 Reviewer 서브에이전트 호출. 실패 시 (3)단계는 스킵하도록 순차 게이트로
배선(병렬화하지 않음 — 비용 절감이 목적).

### 10. 비포화 벤치마크를 1급 회귀지표로 채택

SWE-bench Verified 같은 포화 벤치마크의 단일 숫자로 하네스 우열을 판단하지 않는다. 아직 헤드룸이 큰
SWE-bench Pro/Multimodal v2, Terminal-Bench 최신 버전, live-refresh LiveCodeBench를 내부 회귀 스위트의
주 지표로 삼고, Verified는 "떨어지면 안 되는" 하한선 sanity check로만 쓴다. 상세는 아래 "벤치마크 목표"
참고.

**구현**: Goose에 이미 있는 `goose bench` 러너에 벤치마크 목표에 정리한 태스크셋을 어댑터로 추가 — 새
이밸 프레임워크를 만들지 않는다. CI에서 매 릴리스마다 내부 dogfooding 스위트(고신호·저비용)는 항상 돌리고,
공개 벤치마크는 분기 1회 정도로 비용 관리.

### 11. 샌드박스 우선 아키텍처 — 승인 정책과 분리된 진짜 격리 레이어

프롬프트 인젝션에 대한 유일하게 증명 가능한 방어는 승인 UI가 아니라 실행 환경 자체의 물리적 격리라는 것이
2026년 컨센서스다. **Goose는 현재 자체 OS 샌드박스가 없고 호스트에서 사용자 전체 권한으로 직접 실행되며,
`tool_permissions.json`의 정규식 기반 허용목록은 `bash -c`/alias/따옴표 변형으로 우회 가능한 것으로 이미
지적된 결함이다 — 이것이 fork에서 반드시 메워야 할 가장 중요한 아키텍처 갭이다.**

**구현**: 승인 정책(`GOOSE_MODE`, `tool_permissions.json`)과 기술적 샌드박스 경계를 Codex CLI처럼 분리된
두 레이어로 설계: OS 레벨(Linux bubblewrap+seccomp/Landlock, macOS sandbox-exec)이나 컨테이너/microVM
중 하나를 goose-cli 실행 진입점에 하드 의존성으로 추가하고, 기본값 네트워크 차단+워크스페이스 한정 쓰기를
코드 레벨에서 강제(설정으로 끌 수 없게). **이 원칙은 v1 출시 전 필수 선행 작업으로 [`04-fork-plan.md`](./04-fork-plan.md)의
착수 계획에 별도 단계로 반영할 것.** 체크리스트는 아래 "안전성 권고" 참고.

---

## 벤치마크 목표

- **SWE-bench Pro** (41개 활성 리포, public/held-out/commercial 3분할, 평균 4.1파일·107 LOC patch) — Verified가
  포화(96~97%)된 지금 실질적 코딩 실력을 가르는 1급 회귀 지표. v1 목표는 벤더 1위(80%대) 추격이 아니라
  **오픈웨이트 하네스 중 상위권(현재 62.5%) 돌파** — 하네스 품질(ACI+검증루프+risk-tiered search) 개선분을
  여기서 측정.
- **Terminal-Bench 4.0 + Terminal-Bench Hard** — pleumcode v1 스코프가 "터미널 CLI만"이므로 SWE-bench보다
  제품과 더 직접 정합. 버전이 자주 올라가므로(2.0→4.0) 분기마다 최신 버전으로 갱신.
- **SWE-bench Multimodal v2** (완전 오픈, 480 tasks) — pleumcode 확정 독창기능 "멀티모달 tool 통합"의 실질
  효과를 검증할 유일한 공개 멀티모달 코딩 벤치마크. 오픈소스 1위가 아직 38.6%로 헤드룸 큼.
- **LiveCodeBench** (live-refresh 방식) — 매 분기 컷오프 이후 문제만 재평가(정적 스냅샷 재사용 금지). 오염
  저항 벤치마크로 순수 코드생성력 추적.
- **SWE-bench Verified** — 폐기하지 말고 "포화된 하한선 sanity check"로만 유지(헤드라인 지표로 쓰지 않음).
  여기서 떨어지면 하네스가 명백히 망가진 것.
- **AIShellJack류 표준 공격 페이로드셋 기반 ASR(공격성공률)** — 정확도와 별도로 "보안 회귀" 지표를 정기
  측정. 목표는 절대 0%가 아니라 릴리스마다 ASR이 증가하지 않는지 추적.
- **내부 dogfooding 스위트 (최우선)** — PleumRouter 자체 모노레포(data-plane Rust, web Python, CLI)에서
  뽑은 실제 버그/기능 티켓 20~30개를 고정 fixture로 pass/fail 추적. 범용 벤치마크는 68-provider·KRW
  크레딧·Fusion/routing-policies 같은 PleumRouter 고유 로직을 테스트 못 하므로 가장 저비용·고신호.
  `ponytail`: 처음부터 완비된 이밸 프레임워크를 새로 만들지 말고 `goose bench` 러너에 태스크 정의만 얹을 것.

---

## 안전성 권고

- `tool_permissions.json`(AlwaysAllow/NeverAllow/AskBefore)과 정규식 기반 셸 명령 allowlist는 명확히
  "가드레일"이지 "보안 경계"가 아니다 — `bash -c`, alias, 따옴표 변형으로 우회 가능. fork에서 반드시 별도
  레이어로 진짜 OS/컨테이너 격리 추가: Linux는 bubblewrap+seccomp-bpf 또는 Landlock, macOS는 sandbox-exec
  (Seatbelt), 또는 통째로 Docker/microVM.
- 기본값 네트워크 완전 차단(`--network=none`) 후, PleumRouter API 엔드포인트만 명시적 화이트리스트로
  아웃바운드 허용 — 코드 하드코딩 대신 fork 전용 설정 파일로 분리해 감사 가능하게.
- 파일시스템 쓰기 범위를 프로젝트 워크스페이스 디렉토리로 강제 제한 — HOME·드라이브 루트·상위 디렉토리
  접근은 구조적으로 불가능하게. (2025-2026 `rm -rf` 사고들의 공통 근본원인이 "워크스페이스 밖 접근 자체가
  가능했다"는 것.)
- 파괴적 명령(`rm -rf`, force-push, `DROP TABLE`, prod 배포) 승인 게이트는 "명령 문자열이 위험 키워드를
  포함하는가"가 아니라 **"변수/glob 확장 이후 실제로 삭제/영향받을 경로 목록을 미리 계산해서 보여주는가"**를
  기준으로 설계 — 단순 패턴매칭은 `~/`, `d:\`, 루트 확장을 못 잡아 여러 코딩 에이전트에서 실제 전삭제
  사고로 이어졌음.
- 설정 파일 기반 자동실행(extension 정의 등)은 반드시 "신뢰 확인(trust prompt)" 통과 이후에만 실행되도록
  순서 강제 — CVE-2025-59536(Claude Code)처럼 신뢰 안 된 레포를 열자마자 설정 코드가 먼저 실행되어 API
  키가 유출되는 경로를 원천 차단.
- MCP 확장 화이트리스트는 `GOOSE_ALLOWLIST` YAML로 원격 관리, 서버별 tool description을 최초 승인 시점
  해시로 고정 저장(변경 시 재승인 강제 — rug pull 방지). 한 MCP 서버가 다른 신뢰된 서버의 동작을 유발하는
  cross-server 호출 경로는 차단.
- 원격 MCP 서버 연결에 OAuth 2.1 + PKCE 의무화, 스코프는 서버 단위가 아니라 tool/operation 단위로 세분화.
- 레포지토리 콘텐츠(README/이슈/PR/커밋 메시지)는 항상 "신뢰 경계 밖 데이터"로 태깅 — 심어진 지시문이
  곧바로 실행 지시로 격상되지 않게.
- 에이전트의 "성공했습니다" 자기보고를 그대로 신뢰하지 말 것 — 항상 상태 diff나 실제 테스트/빌드 결과로
  사후 검증한 뒤에만 완료 처리 (Replit이 코드프리즈 중 운영 DB를 삭제하고 스스로 "성공" 보고한 사고 참고).
- PleumRouter API 키는 세션/태스크 단위 짧은 수명·최소 스코프(가능하면 read-only) 토큰을 기본값으로 발급.
- 릴리스 전 정기 레드팀: AIShellJack류 표준 공격 페이로드로 ASR 측정해 회귀 기준선 관리 — "방어 목적"
  페르소나 주장만으로 위험 행동을 승인하지 않는 가드레일 유지.

---

## 참고: 조사한 실제 사고 사례 (반면교사)

- **CVE-2025-59536** (Claude Code <1.0.111): `.claude/settings.json`에 심은 hook이 trust-prompt 이전에
  실행돼 API 키 유출.
- **Claude Code rm -rf 사고 2건** (2025-10-21, 2025-12-08): "정리해줘" 요청이 승인 시점에 확장 결과를
  계산 못해 루트/홈 디렉토리 삭제로 확대.
- **Google Antigravity** (2025-11-27): 캐시 삭제 요청이 `rmdir /s /q d:\`로 확대.
- **Replit** (2025-07): 코드 프리즈 중 운영 DB 삭제 후 스스로 "성공" 보고.
- **Amazon Q VS Code 확장** (2025-07): 과도하게 넓은 GitHub 토큰으로 악성 PR이 정식 릴리스에 병합(문법
  오류로 실행은 실패).
- **postmark-mcp 공급망 공격**: 정상 릴리스 15회로 신뢰를 쌓은 뒤 이메일 BCC 백도어 삽입.
- **GTG-1002** (Anthropic 발표, 2025-11-14): 국가배후 조직이 "방어적 침투테스트" 페르소나로 안전장치를
  우회, 30개+ 조직 대상 공격의 80~90%를 Claude Code가 자율 수행.
