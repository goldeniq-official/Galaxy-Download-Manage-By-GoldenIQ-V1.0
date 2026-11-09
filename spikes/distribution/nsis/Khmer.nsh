;Language: English (1033)
;By Joost Verburg

!insertmacro LANGFILE "Khmer" "Khmer" "ខ្មែរ" "Khmer" ; See \Include\LangFile.nsh for a description of these parameters

!ifdef MUI_WELCOMEPAGE
  ${LangFileString} MUI_TEXT_WELCOME_INFO_TITLE "សូមស្វាគមន៍មកកាន់ការដំឡើង $(^NameDA)"
  ${LangFileString} MUI_TEXT_WELCOME_INFO_TEXT "កម្មវិធីនេះនឹងណែនាំអ្នកក្នុងការដំឡើង $(^NameDA)។$\r$\n$\r$\nការដំឡើងនេះសម្រាប់តែគណនីរបស់អ្នកប៉ុណ្ណោះ ហើយមិនត្រូវការសិទ្ធិអ្នកគ្រប់គ្រងទេ។$\r$\n$\r$\n$_CLICK"
!endif

!ifdef MUI_UNWELCOMEPAGE
  ${LangFileString} MUI_UNTEXT_WELCOME_INFO_TITLE "Welcome to $(^NameDA) Uninstall"
  ${LangFileString} MUI_UNTEXT_WELCOME_INFO_TEXT "Setup will guide you through the uninstallation of $(^NameDA).$\r$\n$\r$\nBefore starting the uninstallation, make sure $(^NameDA) is not running.$\r$\n$\r$\n$_CLICK"
!endif

!ifdef MUI_LICENSEPAGE
  ${LangFileString} MUI_TEXT_LICENSE_TITLE "License Agreement"
  ${LangFileString} MUI_TEXT_LICENSE_SUBTITLE "Please review the license terms before installing $(^NameDA)."
  ${LangFileString} MUI_INNERTEXT_LICENSE_BOTTOM "If you accept the terms of the agreement, click I Agree to continue. You must accept the agreement to install $(^NameDA)."
  ${LangFileString} MUI_INNERTEXT_LICENSE_BOTTOM_CHECKBOX "If you accept the terms of the agreement, click the check box below. You must accept the agreement to install $(^NameDA). $_CLICK"
  ${LangFileString} MUI_INNERTEXT_LICENSE_BOTTOM_RADIOBUTTONS "If you accept the terms of the agreement, select the first option below. You must accept the agreement to install $(^NameDA). $_CLICK"
!endif

!ifdef MUI_UNLICENSEPAGE
  ${LangFileString} MUI_UNTEXT_LICENSE_TITLE "License Agreement"
  ${LangFileString} MUI_UNTEXT_LICENSE_SUBTITLE "Please review the license terms before uninstalling $(^NameDA)."
  ${LangFileString} MUI_UNINNERTEXT_LICENSE_BOTTOM "If you accept the terms of the agreement, click I Agree to continue. You must accept the agreement to uninstall $(^NameDA)."
  ${LangFileString} MUI_UNINNERTEXT_LICENSE_BOTTOM_CHECKBOX "If you accept the terms of the agreement, click the check box below. You must accept the agreement to uninstall $(^NameDA). $_CLICK"
  ${LangFileString} MUI_UNINNERTEXT_LICENSE_BOTTOM_RADIOBUTTONS "If you accept the terms of the agreement, select the first option below. You must accept the agreement to uninstall $(^NameDA). $_CLICK"
!endif

!ifdef MUI_LICENSEPAGE | MUI_UNLICENSEPAGE
  ${LangFileString} MUI_INNERTEXT_LICENSE_TOP "Press Page Down to see the rest of the agreement."
!endif

!ifdef MUI_COMPONENTSPAGE
  ${LangFileString} MUI_TEXT_COMPONENTS_TITLE "Choose Components"
  ${LangFileString} MUI_TEXT_COMPONENTS_SUBTITLE "Choose which features of $(^NameDA) you want to install."
!endif

!ifdef MUI_UNCOMPONENTSPAGE
  ${LangFileString} MUI_UNTEXT_COMPONENTS_TITLE "Choose Components"
  ${LangFileString} MUI_UNTEXT_COMPONENTS_SUBTITLE "Choose which features of $(^NameDA) you want to uninstall."
!endif

!ifdef MUI_COMPONENTSPAGE | MUI_UNCOMPONENTSPAGE
  ${LangFileString} MUI_INNERTEXT_COMPONENTS_DESCRIPTION_TITLE "Description"
  !ifndef NSIS_CONFIG_COMPONENTPAGE_ALTERNATIVE
    ${LangFileString} MUI_INNERTEXT_COMPONENTS_DESCRIPTION_INFO "Position your mouse over a component to see its description."
  !else
    ${LangFileString} MUI_INNERTEXT_COMPONENTS_DESCRIPTION_INFO "Select a component to see its description."
  !endif
!endif

!ifdef MUI_DIRECTORYPAGE
  ${LangFileString} MUI_TEXT_DIRECTORY_TITLE "ជ្រើសទីតាំងដំឡើង"
  ${LangFileString} MUI_TEXT_DIRECTORY_SUBTITLE "ជ្រើសថតដែលត្រូវដំឡើង $(^NameDA)។"
!endif

!ifdef MUI_UNDIRECTORYPAGE
  ${LangFileString} MUI_UNTEXT_DIRECTORY_TITLE "Choose Uninstall Location"
  ${LangFileString} MUI_UNTEXT_DIRECTORY_SUBTITLE "Choose the folder from which to uninstall $(^NameDA)."
!endif

!ifdef MUI_INSTFILESPAGE
  ${LangFileString} MUI_TEXT_INSTALLING_TITLE "កំពុងដំឡើង"
  ${LangFileString} MUI_TEXT_INSTALLING_SUBTITLE "សូមរង់ចាំ ខណៈពេល $(^NameDA) កំពុងត្រូវបានដំឡើង។"
  ${LangFileString} MUI_TEXT_FINISH_TITLE "ការដំឡើងបានបញ្ចប់"
  ${LangFileString} MUI_TEXT_FINISH_SUBTITLE "ការដំឡើងបានបញ្ចប់ដោយជោគជ័យ។"
  ${LangFileString} MUI_TEXT_ABORT_TITLE "ការដំឡើងត្រូវបានបញ្ឈប់"
  ${LangFileString} MUI_TEXT_ABORT_SUBTITLE "ការដំឡើងមិនបានបញ្ចប់ដោយជោគជ័យទេ។"
!endif

!ifdef MUI_UNINSTFILESPAGE
  ${LangFileString} MUI_UNTEXT_UNINSTALLING_TITLE "កំពុងលុប"
  ${LangFileString} MUI_UNTEXT_UNINSTALLING_SUBTITLE "សូមរង់ចាំ ខណៈពេល $(^NameDA) កំពុងត្រូវបានលុប។"
  ${LangFileString} MUI_UNTEXT_FINISH_TITLE "ការលុបបានបញ្ចប់"
  ${LangFileString} MUI_UNTEXT_FINISH_SUBTITLE "ការលុបបានបញ្ចប់ដោយជោគជ័យ។"
  ${LangFileString} MUI_UNTEXT_ABORT_TITLE "Uninstallation Aborted"
  ${LangFileString} MUI_UNTEXT_ABORT_SUBTITLE "Uninstall was not completed successfully."
!endif

!ifdef MUI_FINISHPAGE
  ${LangFileString} MUI_TEXT_FINISH_INFO_TITLE "កំពុងបញ្ចប់ការដំឡើង $(^NameDA)"
  ${LangFileString} MUI_TEXT_FINISH_INFO_TEXT "$(^NameDA) ត្រូវបានដំឡើងលើកុំព្យូទ័ររបស់អ្នករួចហើយ។$\r$\n$\r$\nចុច បញ្ចប់ ដើម្បីបិទ។"
  ${LangFileString} MUI_TEXT_FINISH_INFO_REBOOT "Your computer must be restarted in order to complete the installation of $(^NameDA). Do you want to reboot now?"
!endif

!ifdef MUI_UNFINISHPAGE
  ${LangFileString} MUI_UNTEXT_FINISH_INFO_TITLE "Completing $(^NameDA) Uninstall"
  ${LangFileString} MUI_UNTEXT_FINISH_INFO_TEXT "$(^NameDA) has been uninstalled from your computer.$\r$\n$\r$\nClick Finish to close Setup."
  ${LangFileString} MUI_UNTEXT_FINISH_INFO_REBOOT "Your computer must be restarted in order to complete the uninstallation of $(^NameDA). Do you want to reboot now?"
!endif

!ifdef MUI_FINISHPAGE | MUI_UNFINISHPAGE
  ${LangFileString} MUI_TEXT_FINISH_REBOOTNOW "Reboot now"
  ${LangFileString} MUI_TEXT_FINISH_REBOOTLATER "I want to manually reboot later"
  ${LangFileString} MUI_TEXT_FINISH_RUN "&បើក $(^NameDA)"
  ${LangFileString} MUI_TEXT_FINISH_SHOWREADME "&Show Readme"
  ${LangFileString} MUI_BUTTONTEXT_FINISH "&បញ្ចប់"  
!endif

!ifdef MUI_STARTMENUPAGE
  ${LangFileString} MUI_TEXT_STARTMENU_TITLE "Choose Start Menu Folder"
  ${LangFileString} MUI_TEXT_STARTMENU_SUBTITLE "Choose a Start Menu folder for the $(^NameDA) shortcuts."
  ${LangFileString} MUI_INNERTEXT_STARTMENU_TOP "Select the Start Menu folder in which you would like to create the program's shortcuts. You can also enter a name to create a new folder."
  ${LangFileString} MUI_INNERTEXT_STARTMENU_CHECKBOX "Do not create shortcuts"
!endif

!ifdef MUI_UNCONFIRMPAGE
  ${LangFileString} MUI_UNTEXT_CONFIRM_TITLE "លុប $(^NameDA)"
  ${LangFileString} MUI_UNTEXT_CONFIRM_SUBTITLE "លុប $(^NameDA) ចេញពីកុំព្យូទ័ររបស់អ្នក។"
!endif

!ifdef MUI_ABORTWARNING
  ${LangFileString} MUI_TEXT_ABORTWARNING "តើអ្នកពិតជាចង់ចាកចេញពីការដំឡើង $(^Name) មែនទេ?"
!endif

!ifdef MUI_UNABORTWARNING
  ${LangFileString} MUI_UNTEXT_ABORTWARNING "តើអ្នកពិតជាចង់ចាកចេញពីការលុប $(^Name) មែនទេ?"
!endif

!ifdef MULTIUSER_INSTALLMODEPAGE
  ${LangFileString} MULTIUSER_TEXT_INSTALLMODE_TITLE "Choose Users"
  ${LangFileString} MULTIUSER_TEXT_INSTALLMODE_SUBTITLE "Choose for which users you want to install $(^NameDA)."
  ${LangFileString} MULTIUSER_INNERTEXT_INSTALLMODE_TOP "Select whether you want to install $(^NameDA) only for yourself or for all users of this computer. $(^ClickNext)"
  ${LangFileString} MULTIUSER_INNERTEXT_INSTALLMODE_ALLUSERS "Install for anyone using this computer"
  ${LangFileString} MULTIUSER_INNERTEXT_INSTALLMODE_CURRENTUSER "Install just for me"
!endif
