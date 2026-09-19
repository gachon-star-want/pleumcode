# Fork 대상 조사 & 착수 계획

> 2026-09-18 워크플로우 리서치(`wf_98015d23-d89`) 결과 요약. 후보 9개(opencode·OpenHands·Aider·Goose·Crush·
> Kilo Code·Hermes Agent·OpenClaw·직접구축)를 6개 기준(라이선스/런타임 궁합/provider 모듈성/tool-use 성숙도/
> 활동성/통합난이도)으로 병렬 조사 후 종합 판정.

## 종합 순위

| 순위 | 후보 | 점수(10점) | 한줄 이유 |
|---|---|---|---|
| 1 | **Goose** (block/goose, Rust) | 8.5 | Provider trait + JSON 설정 파일만으로 core 무수정 연결. MCP/샌드박스/컨텍스트압축 등 tool-use 루프 최고. Apache-2.0 + Linux Foundation 이관으로 라이선스·버스팩터 리스크 최저 |
| 2 | opencode (anomalyco/opencode, TS) | 7.6 | 이미 `pleum launch opencode`로 wiring 검증 끝난 상태. TS/Node라 팀 스택과 궁합 좋음. 단점: v1.0부터 Bun+Zig(OpenTUI) 네이티브 빌드 |
| 3 | Kilo Code | 7.2 | UX 완성도 최고지만 opencode의 "fork의 fork" — 업스트림 추적 이중 |
| 4 | OpenHands | 6.0 | LiteLLM 기반이라 연결은 쉬우나 SDK/CLI/프론트 3개 저장소로 분산 |
| 5 | 직접 구축(`run.mjs` 확장) | 3.7 | tool-use 루프(diff apply, 권한, 샌드박스, MCP)를 처음부터 짜는 건 몇 개월~분기 프로젝트 — 비권장 |
| 6 | Aider | 3.3 | Apache-2.0은 깨끗하나 모놀리식 구조 + 메인테이너 활동 둔화 신호 |
| 7 | Hermes Agent | 3.0 | 범용 어시스턴트 — 코딩은 자체 문서조차 OpenCode에 위임한다고 명시 |
| 8 | OpenClaw | 1.5 | 코딩 CLI 아님(멀티채널 게이트웨이 데몬). 자사 `cli/src/recipes.mjs`가 이미 같은 이유로 배제한 기록 있음 |
| 9 | Crush | 1.0 | ⚠️ FSL-1.1-MIT의 "Competing Use" 조항이 코딩 에이전트 상품화와 충돌 위험 — fork 대상 제외 |

## 왜 Goose인가

- Smart Mode/Fusion/routing-policies는 게이트웨이가 이미 서버사이드에서 처리하고 `policy/<slug>`·`orch/<slug>` 같은
  **모델 ID 문자열**로 노출하는 구조라, Goose의 provider 계층(`~/.config/goose/providers/*.json`)에 그 슬러그를
  나열하기만 하면 core를 한 줄도 안 건드리고 그대로 동작한다.
- "Rust는 팀에 낯선 런타임"이라는 우려는 사실이 아님 — `PleumRouter/data-plane`가 이미 axum/tokio 기반 Rust
  workspace로 프로덕션 운영 중임을 실측 확인.
- MCP 1급 지원, 프로젝트 범위 샌드박스, 토큰 80% 도달 시 자동 컨텍스트 압축("Smart Context Management") 등
  tool-use 루프가 조사된 9개 후보 중 가장 성숙.
- Apache-2.0 + Block에서 Linux Foundation 산하 Agentic AI Foundation으로 거버넌스 이관 중 — 특정 회사 종속
  리스크가 낮음(단, 이관 정확한 범위는 fork 착수 직전 재확인 필요 — 아래 리스크 참고).

대안(2순위): **opencode 직접 fork** — 이미 `cli/src/recipes.mjs`가 `OPENCODE_CONFIG_CONTENT`로 launch 검증까지
마쳐서 시작 리스크가 가장 낮고, 프론트엔드/CLI 팀이 익숙한 TS 계열. KRW 비용 표시를 "상시 상태바"가 아니라
"턴 종료 후 한 줄 출력"으로 스코프를 좁히면 Zig/OpenTUI core를 깊이 건드릴 필요가 줄어든다. 팀이 Rust보다 TS를
선호하게 되면 이쪽으로 전환 검토.

## 착수 계획 (8단계)

1. **라이선스/조직 재확인**(반나절): `block/goose` vs 재단 이관된 조직 중 현재 정본을 재확인하고, 실제 fork
   커밋에 Apache-2.0 라이선스 원문을 스냅샷. main이 아니라 특정 태그에 고정해서 시작.
2. **core 무수정 검증**: `~/.config/goose/providers/pleum.json` 하나로 PleumRouter를 provider 등록
   (`host=router.pleum.ai`, `engine=openai`, Authorization 헤더에 `${PLEUM_API_KEY}`)하고 `models` 배열에
   일반 모델 슬러그·`smart`·`policy/<slug>`·`orch/<slug>`를 나열 — core 한 줄도 안 고치고 기존
   `routing_policies.py`·`fusion.py`가 서빙하는 라우팅이 그대로 동작하는지부터 확인.
3. 이 provider json을 기존 `cli/src/recipes.mjs`의 `RECIPES` 패턴(`type:'env'`, `env:(key)=>({...})`)에 맞춰
   새 레시피로 편입 — 기존 10개 레시피·MANUAL(crush) 처리는 그대로 공존.
4. `goose-cli` 크레이트에 한정해 세션 종료/턴 완료 훅 지점에 **KRW 비용 한 줄 출력** 추가(상시 상태바 아님,
   v1은 print 한 줄로 스코프 고정) — PleumRouter 응답의 usage/cost 필드 사용.
5. **routing-policy 피커**를 슬래시 명령(`/policy`, `/smart`)으로 노출 — 게이트웨이의 routing-policies 목록
   엔드포인트를 호출해 raw 모델 문자열 대신 사람이 읽을 수 있는 정책 이름으로 선택.
6. **Fusion**(2~4모델 병렬+합성)을 `--fusion` 플래그/전용 모드로 추가 — `orch/<slug>` 호출 결과를 모델별
   블록 + 합성 결과로 렌더링. 유일하게 실질적인 신규 Rust UI 코드가 필요한 지점이라 4~5단계 이후로 미룸.
7. 업스트림 goose에 대한 리베이스 주기(예: 월 1회 태그 추적) 확정 — fork를 "1회성 복사"가 아니라 지속 비용으로
   예산에 반영.
8. 신규 바이너리는 기존 `pleum` 런처와 별도 배포(예: `pleum-code`)로 시작해 opt-in 베타로 굴리고, 안정화된
   뒤에만 `pleum launch`의 기본 타깃으로 승격 검토 — 기존 10개 레시피 사용자에게 회귀 없도록 병행 운영.

## ⚠️ 추가 확정 사항 (2026-09-19, 하네스 리서치)

[`03-harness-design.md`](./03-harness-design.md) 리서치에서 **Goose는 자체 OS 샌드박스가 없고 호스트에서
사용자 전체 권한으로 직접 실행되며, `tool_permissions.json`의 정규식 기반 셸 명령 allowlist는 `bash -c`/
alias/따옴표 변형으로 우회 가능한 "가드레일일 뿐 보안 경계가 아니다"**라는 사실을 확인했다. pleumcode는
유료 KRW 크레딧으로 임의 명령을 실행하는 제품이므로, **이 갭을 메우는 것을 위 8단계보다 먼저(2단계 직후,
3단계 착수 전) 처리하는 별도 단계로 승격한다**: OS 커널 샌드박스(Linux bubblewrap+seccomp/Landlock, macOS
sandbox-exec)나 컨테이너/microVM 중 하나를 goose-cli 실행 진입점에 하드 의존성으로 추가하고, 기본값
네트워크 차단+워크스페이스 한정 쓰기를 설정으로 끌 수 없게 코드 레벨에서 강제한다. 체크리스트는
[`03-harness-design.md`](./03-harness-design.md)의 "안전성 권고" 참고.

또한 서브에이전트별 다른 모델 지정([`01-unique-features.md`](./01-unique-features.md) 핵심 기능)에 관련해
**Goose에 알려진 업스트림 버그**(`GOOSE_SUBAGENT_MODEL` 환경변수를 전역 설정하면 recipe별 role-specific
모델 지정이 collapse됨, 2026-09 `aaif-goose/goose#11862`)가 있음을 fork 착수 시 재확인하고, 고쳐지지 않았다면
우리 fork에서 먼저 패치할 것.

## 핵심 리스크

- **Fork 유지보수 세금**: 매 upstream goose 릴리스(신규 MCP 툴, 모델, 보안 패치)를 커스텀 Rust CLI 변경 위에
  수동 병합/리베이스해야 함 — 가장 과소평가되기 쉬운 비용.
- **거버넌스/라이선스 재확인 필요**: `block/goose` → 재단 이관의 정확한 시점·범위가 리서치 시점에도
  확인불가였음 — fork 실행 직전 다시 확인.
- **두 번째 Rust 릴리스 파이프라인**: `data-plane`과는 별도 바이너리·CI·배포 채널이 추가로 필요.
- **서버사이드 로직 중복 금지**: Smart Mode/Fusion/routing-policies 판단 로직을 Rust 포크에 절대 재구현하지
  말 것 — 모델ID 패스스루로만 유지해야 진실 소스가 두 개로 갈라지는 걸 막을 수 있음.
- **스코프 크립**: KRW 비용/Fusion 렌더링을 "한 줄 출력"으로 시작해도 요구가 커지면 결국 TUI 내부를 깊이
  고치는 프로젝트로 번질 수 있음 — v1 스코프를 문서로 못박아 둘 것.
- **경쟁 포지셔닝**: BizRouter는 이미 Claude Code/Codex/Cline/OpenCode 4종 통합을 갖춤 — 미완성 상태로
  서둘러 전면 배포하면 오히려 기존 10-에이전트 런처보다 못해 보일 수 있음 → opt-in 베타로 단계적 출시.
- **정보 신선도**: stars/이슈 수/라이선스 조항은 계속 바뀌므로 실제 fork 착수 직전 재검증 필요.

## 진행 로그 — v0.1 착수 (2026-09-19)

### 0~1단계: 완료 (브랜치 `feat/v0.1-bringup`, 미푸시)

- **정본 조직 확정**: `block/goose`는 `aaif-goose/goose`로 리다이렉트된다(Apache-2.0, archived 아님). 이후 모든
  참조는 `aaif-goose/goose`.
- **고정 태그**: `v1.51.0`(2026-09-17 stable). `v2-rc*` 태그는 RC라 제외.
- **방식**: 기존 `pleumcode` 저장소(문서 보유)에 `upstream` remote를 추가하고 `--allow-unrelated-histories`로
  v1.51.0을 병합 — 업스트림 히스토리가 보존되어 7단계(월 1회 리베이스)의 merge-base가 생긴다. 충돌 3건:
  LICENSE는 업스트림 것(`Copyright 2024 Block, Inc.` 표기 포함) 채택, README는 우리 것, `.gitignore`는 합침.
- pleumcode 전용 파일은 `pleum/` 아래에 모아 업스트림 트리와 분리.

### 2단계: 배선 검증 완료, 200 응답은 미확인

**본 문서 위쪽 2단계 설명의 오류 정정** (v1.51.0 실제 스키마 기준):

| 항목 | 위 본문 | 실제 |
|---|---|---|
| 위치 | `~/.config/goose/providers/pleum.json` | `~/.config/goose/custom_providers/pleum.json` |
| 호스트 | `router.pleum.ai` | API는 `https://apirouter.pleum.ai/v1` (`router.pleum.ai`는 사이트 도메인) |
| 인증 | Authorization 헤더에 `${PLEUM_API_KEY}` | `api_key_env: "PLEUM_API_KEY"` |
| 필드 | `host` | `base_url`, `engine: "openai"` |

- 더미 키로 실호출: `pleum` provider가 core 무수정으로 로드되고, 요청이 게이트웨이(`/v1/responses`)까지
  도달해 게이트웨이 자체의 401을 받았다. **진짜 키로 200 응답과 `policy/<slug>`·`orch/<slug>` 패스스루는 아직
  검증 전.** Goose는 이 모델에 Chat Completions가 아니라 Responses API를 쓰므로 게이트웨이의
  `/v1/responses` 스트리밍 호환도 그때 함께 확인해야 한다.
- 정적 `models`에는 검증된 `gpt-5.4-mini`만 넣었고 나머지는 `/v1/models` 동적 조회에 맡긴다(`smart`
  슬러그의 실재는 미확인이라 넣지 않음).

### `GOOSE_SUBAGENT_MODEL` 버그(`aaif-goose/goose#11862`) 재확인

- v1.51.0 코드(`summon.rs:1726`)에는 **여전히 버그가 남아 있다**(env가 `recipe.settings`보다 우선).
- 이슈는 PR #12070(머지 `9222d96`, 2026-09-16, `summon.rs` +87/-38)으로 닫혔으나 v1.51.0 태그에는 미포함.
  v0.3 착수 전 다음 업스트림 릴리스에 들어왔는지 확인하고, 없으면 해당 커밋을 cherry-pick한다(지금은
  리베이스 충돌만 늘리므로 미적용).

### 03 문서 구현 위치 정정

03의 "goose-mcp developer extension" 표현은 v1.51.0에서 틀렸다. 셸/편집 도구는
`crates/pleum/src/agents/platform_extensions/developer/`에(업스트림 v1.51.0에선 `crates/goose/...`), 서브에이전트 위임은 `summon` platform extension
(`platform_extensions/summon.rs`)에 있다. v0.2 verify wrapper와 v0.3 디스패치는 이 위치를 기준으로 설계한다.

### OS 샌드박스 v0.1 첫 컷 (macOS만)

설계는 Codex 방식의 분리: **에이전트 프로세스는 그대로 두고(모델 API 통신 필요), 도구가 spawn하는 명령만
샌드박스에 넣는다.** 런타임 opt-out 없음(env·config 키 없음).

- `crates/pleum/src/sandbox.rs` — `wrap()`(Seatbelt `sandbox-exec`, 경로는 `-D` 파라미터로 전달해 프로파일에
  문자열 삽입 없음)과 `check_write()`(프로세스 내부 쓰기용 동일 정책).
- 정책: 쓰기는 canonical 워크스페이스 + 임시 디렉토리만 / 네트워크 전면 차단 / `.git/hooks`·`.git/config`
  쓰기 금지(나중에 호스트에서 실행되는 탈출 경로) / `~/.ssh`·`~/.aws`·`~/.gnupg`·goose 설정 디렉토리
  읽기 금지(출력이 곧 모델 컨텍스트로 가므로).
- 연결: `developer/shell.rs`의 `run_command`(셸 도구)와 `developer/edit.rs`의 `write`/`edit`. 후자는 goose
  프로세스 안에서 도는 도구라 Seatbelt가 못 막으므로 `check_write`로 별도 가드(디렉토리 생성 **전에** 검사).
  심볼릭 링크·dangling 링크·`..` 탈출 모두 차단, 테스트로 검증.
- **미지원 OS는 fail-closed**: 명령을 실행하지 않고 에러를 낸다.

**아직 안 막힌 것 (v1.0-beta 전 필수)**

- Linux(bubblewrap+Landlock)·Windows 백엔드 — 지금 두 OS에서는 셸 도구가 동작하지 않는다.
- `hooks/mod.rs`의 `sh -c`, stdio MCP 확장 spawn(`extension_manager.rs`), `goose-mcp`의 자체 spawn/파일
  쓰기는 샌드박스 밖. 특히 hooks는 CVE-2025-59536과 같은 종류라 trust prompt 순서 점검과 함께 다뤄야 한다.
- 자식 프로세스가 환경변수(`PLEUM_API_KEY` 등)를 그대로 상속 — `env`만 쳐도 키가 모델 컨텍스트로 간다.
- 네트워크 전면 차단이라 자식의 `npm install`/`git fetch`/`cargo fetch`가 실패한다(PleumRouter 화이트리스트는
  에이전트 프로세스 쪽이라 해당 없음). 허용 정책(프록시+allowlist)은 별도 결정이 필요하다.
- 업스트림 기본 feature에 `telemetry`·`otel`·`nostr`·`update`가 켜져 있다 — 배포 전 끄거나 점검.

### 컨텍스트 파일: AGENTS.md 최우선 (채택순서 1번, 2026-09-19)

사용자가 이미 `AGENTS.md`/`CLAUDE.md`를 갖고 있으므로 새 파일을 강요하지 않고 그대로 읽는다.
**이 결정으로 02의 `PLEUM.md`와 03 원칙 4의 `pleumcode.md`(둘 사이 이름도 불일치였음)는 만들지 않는다** —
pleumcode 전용 규칙이 실제로 필요해질 때까지 YAGNI.

- 업스트림 v1.51.0 기본값은 `[".goosehints", "AGENTS.md"]`였고 `CLAUDE.md`는 읽지 않았다.
- pleumcode 기본값: `["AGENTS.md", "CLAUDE.md", ".pleumhints"]`(`hints/load_hints.rs`).
  `CONTEXT_FILE_NAMES`로 덮어쓰면 그 값을 그대로 따른다.
- **`CLAUDE.md`는 같은 디렉토리에 `AGENTS.md`가 없을 때만 읽는다.** 실제 저장소의 `CLAUDE.md`는 대개
  `@AGENTS.md` 한 줄 포인터나 심볼릭 링크라서, 둘 다 읽으면 같은 내용이 프롬프트에 두 번 들어간다(토큰=KRW).
  fallback은 디렉토리 단위라 루트는 AGENTS.md, 하위는 CLAUDE.md만 있는 모노레포도 정상 동작한다.
- 의도적으로 하지 않은 것: 사용자 홈의 `~/.claude/CLAUDE.md`를 글로벌 힌트로 읽는 것(다른 도구의 개인 설정을
  몰래 가져오는 셈). 필요하면 사용자가 `~/.agents/AGENTS.md`에 두면 된다.
- 레포 안의 이 파일들은 신뢰 경계 밖 데이터다(03 안전성 권고). 로딩 대상이 넓어진 게 아니라 이미 로드하던
  `AGENTS.md`와 같은 종류라 표면은 그대로지만, 프롬프트 인젝션 태깅은 v0.2에서 별도로 점검한다.

### goose → pleum 전면 리네임 (2026-09-19)

코드·파일명·식별자에서 goose 흔적을 걷어냈다. **이 문서의 위쪽 섹션이 `goose`로 적은 코드 경로·이름은 리네임
이전(=업스트림) 기준**이며, 현재 fork에서는 아래 표로 대응한다.

| 업스트림 | 현재 |
|---|---|
| crate `goose*` (`crates/goose-cli` …) | `pleum*` (`crates/pleum-cli` …) |
| 바이너리 `goose` | **`pleumcode`** (기존 npm 런처가 `pleum`을 쓰므로 충돌 회피, 저장소 이름과도 일치) |
| 환경변수 `GOOSE_*` | `PLEUM_*` (예: `PLEUM_PATH_ROOT`, `PLEUM_MODE`) |
| `~/.config/goose` 등 | `~/.config/pleum`, `~/.local/share/pleum` |
| `.goosehints` / `.gooseignore` | `.pleumhints` / `.pleumignore` |
| `aaif-goose/goose`, `block/goose` | `gachon-star-want/pleumcode` |
| `goose-docs.ai` | `docs.pleum.ai` (**자리표시자** — 우리 도메인 하위지만 사이트는 아직 없음) |

**방식**: `pleum/rebrand.py`(결정적·멱등)가 내용 1,261개 파일과 경로 1,031개를 바꿨다. 저장소 슬러그와
도메인은 일부러 명시 매핑했다 — 그냥 `goose→pleum`이면 `aaif-pleum/pleum` 같은 **우리가 소유하지 않은 org**가
생겨 update/install 경로가 선점될 수 있다.

**일부러 안 바꾼 것**: `LICENSE`(+ `NOTICE` 추가: Apache-2.0 attribution·변경 고지), `docs/`·`README.md`(우리
문서, 업스트림 출처를 서술), crates.io의 **`v8-goose` crate**(실제 외부 의존성이라 바꾸면 빌드가 깨짐,
`vendor/v8` 포함), git remote `upstream`.

**검증**: 잔여 `goose`는 위 제외 대상뿐 / 워크스페이스 전체 `--all-targets` 컴파일 통과 / lib 테스트
2,266 통과·7 실패 — **리네임 이전 커밋에서도 동일한 7개가 동일하게 실패**(최소 feature 빌드의
`jsonwebtoken` CryptoProvider 부재 등), 리네임이 만든 실패는 0 / 컴파일된 바이너리에 goose 문자열 0개 /
더미 키로 게이트웨이 401 재확인(provider 배선 정상).

**업스트림 병합 절차** (모든 파일이 바뀌었으므로 이걸 지키지 않으면 병합마다 전 파일이 충돌한다 — *아직 한 번도
실행해 보지 않음, v1.52 때 첫 검증*):
1. 기준점: `v1.51.0`에 스크립트를 적용해 브랜치 `rebranded-upstream`을 만든다(업스트림 히스토리 보존).
2. 새 릴리스 `vX`: 그 브랜치에서 트리를 `vX`로 교체 → `python3 pleum/rebrand.py .` → 커밋. 이 커밋의 diff가
   곧 "리네임된 순수 업스트림 변경"이다.
3. `main`에 `rebranded-upstream`을 merge. 양쪽이 같은 리네임을 했으므로 진짜 충돌만 남는다.

**후속 결정 (2026-09-19, 모두 반영 완료)**
- **바이너리는 `pleumcode`**: `[[bin]]`, clap `name`, `term.rs`가 실행하는 `pleumcode term ...`, 매뉴얼 생성기, 빌드
  스크립트의 `--bin`·`target/*/pleumcode`·`.exe`까지 `rebrand.py`의 2단계 규칙이 처리한다(명령 형태의 `pleum <서브명령>`
  만 바꾸고 crate·디렉토리 이름 `pleum-cli`는 건드리지 않음). 설정 디렉토리와 환경변수 접두어는 그대로 `pleum`
  (`~/.config/pleum`, `PLEUM_*`) — 런처가 쓰는 `PLEUM_API_KEY` 등과 이름이 겹치는 변수는 없음을 확인했다.
- **`ui/`(Electron)·`documentation/`(Docusaurus)와 그것만 위한 워크플로우 6개 삭제**: v1이 터미널 CLI 전용이라서.
  삭제 목록이 `rebrand.py`(`DELETE`)에 들어 있어 업스트림 병합 후에도 삭제된 채로 유지된다.
- **시작 배너**: ASCII 거위를 없애고 `pleum` 워드마크로 교체. 색은 PleumRouter 런처와 같은 브랜드 plum
  `#936c89` 하나만 포인트로 쓰고 truecolor → xterm-256(139) → 16색 마젠타로 폴백, `NO_COLOR`·파이프에서는
  색 없음(실제 렌더링으로 4가지 모드 확인). 거위 이모지 🪿도 제거(터미널 제목은 `pleum <디렉토리>`).

**아직 남은 것**
- 업스트림이 쓴 기여자 문서(`CONTRIBUTING.md`, `BUILDING_*.md`, `CUSTOM_DISTROS.md`, `RISCV_SETUP.md`)와 `Justfile`의
  데스크톱 레시피, 남긴 워크플로우(`ci.yml`, `build-cli-linux.yml`, `release.yml` 등)에 삭제된 `ui/`·데스크톱 언급이
  남아 있다. 프롬프트에 로드되는 `AGENTS.md`·`.pleumhints`는 이미 정리함. CI/패키징은 우리 배포 파이프라인이 생길
  때(v1.0-beta) 함께 정리하고, 그 전까지는 동작을 보증하지 않는다.
- `--version`이 여전히 업스트림 버전 `1.51.0`을 표시한다 — 새 배포이므로 `0.1.0`부터 다시 시작해야 한다.
- goose가 아닌 상류 흔적: `@aaif/*` npm scope, `Block` 표기, PostHog 등 텔레메트리 키(샌드박스 잔여 목록의
  `telemetry` 항목과 같은 건).
