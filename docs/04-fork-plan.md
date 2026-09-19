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
